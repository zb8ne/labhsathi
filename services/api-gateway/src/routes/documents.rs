use crate::kafka::publish_job_submitted;
use crate::redis_cache;
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use labhsathi_core::events::{DocumentJobSubmitted, JobId};
use labhsathi_core::media::validate_upload;
use labhsathi_core::status::{JobProgress, JobStatusRecord};

fn error(status: StatusCode, message: impl AsRef<str>) -> axum::response::Response {
    (status, Json(serde_json::json!({"error": message.as_ref()}))).into_response()
}

/// Accepts a single-image multipart upload. Validates and writes the image
/// to Redis, publishes `document.jobs.submitted`, and returns immediately
/// with a job_id -- the actual vision-API call happens out-of-band in
/// ocr-worker. The image bytes never leave this function except into Redis
/// with a 60s TTL; they are not logged, not written to disk, not returned.
pub async fn documents_upload_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => return error(StatusCode::BAD_REQUEST, "no file field found"),
            Err(e) => {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("could not read upload: {e}"),
                )
            }
        };

        if field.file_name().is_none() {
            continue; // plain text field, not the upload
        }
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("could not read file contents: {e}"),
                )
            }
        };

        let mime_type = match validate_upload(&data, &content_type) {
            Ok(m) => m,
            Err(msg) => return error(StatusCode::BAD_REQUEST, msg),
        };

        let job_id = JobId::new();
        // GitHub issue #4 (product review): `state.redis` is a plain,
        // cheaply-`Clone`-able `ConnectionManager` now, not a
        // `Mutex`-guarded one -- see state.rs's doc comment. Cloning it
        // here is a handle copy, not a new connection, and (unlike the old
        // Mutex) doesn't block every other in-flight request's Redis
        // access for the duration of this handler, including the Kafka
        // publish a few lines down.
        let mut redis = state.redis.clone();

        if let Err(e) = redis_cache::store_image(&mut redis, job_id, &data).await {
            tracing::error!(%job_id, error = %e, "failed to store image in redis");
            return error(StatusCode::INTERNAL_SERVER_ERROR, "could not accept upload");
        }

        let event = DocumentJobSubmitted {
            job_id,
            mime_type,
            ts: now_millis(),
        };

        if let Err(e) = publish_job_submitted(&state.kafka_producer, &event).await {
            // Compensating action: don't leave an orphaned image in Redis
            // with no worker ever told to look for it.
            tracing::error!(%job_id, error = %e, "kafka publish failed, rolling back redis write");
            redis_cache::delete_image(&mut redis, job_id).await;
            return error(
                StatusCode::SERVICE_UNAVAILABLE,
                "document queue is unavailable, please try again",
            );
        }

        let record = JobStatusRecord {
            status: JobProgress::Queued,
            fields: None,
            error: None,
        };
        if let Err(e) = redis_cache::set_status(&mut redis, job_id, &record).await {
            // Non-fatal: the job is already queued and will still be
            // processed; the status endpoint will just 404 until ocr-worker
            // writes the first status update itself.
            tracing::warn!(%job_id, error = %e, "failed to write initial status record");
        }

        return (StatusCode::ACCEPTED, Json(serde_json::json!({ "job_id": job_id.to_string() })))
            .into_response();
    }
}

pub async fn documents_status_handler(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let job_id: JobId = match job_id.parse() {
        Ok(id) => id,
        Err(_) => return error(StatusCode::BAD_REQUEST, "invalid job id"),
    };

    let mut redis = state.redis.clone();
    match redis_cache::get_status(&mut redis, job_id).await {
        Ok(Some(record)) => Json(record).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "unknown or expired job id -- it may have already completed and aged out, or never existed",
        ),
        Err(e) => {
            tracing::error!(%job_id, error = %e, "redis read failed");
            error(StatusCode::INTERNAL_SERVER_ERROR, "could not read job status")
        }
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
