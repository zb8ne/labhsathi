mod ocr;
mod schemes;

use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use schemes::{match_schemes, SchemeMatch, UserProfile};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

/// Axum defaults to a 2 MB request body, which silently rejects almost every
/// photo taken on a phone. Raise it to match the ceiling enforced in ocr.rs.
const MAX_UPLOAD_BYTES: usize = 20 * 1024 * 1024;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        tracing::warn!(
            "ANTHROPIC_API_KEY is not set — scheme matching works, \
             but document auto-fill will return an error"
        );
    }

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/match", post(match_handler))
        .route(
            "/api/extract-document",
            post(extract_handler).layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
        .fallback_service(ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("scheme-setu listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn match_handler(Json(profile): Json<UserProfile>) -> impl IntoResponse {
    let matches: Vec<SchemeMatch> = match_schemes(&profile);
    Json(serde_json::json!({
        "matched_count": matches.len(),
        "schemes": matches,
    }))
}

fn error(status: StatusCode, message: impl AsRef<str>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({"error": message.as_ref()})),
    )
        .into_response()
}

/// Accepts a single-image multipart upload, extracts structured fields via
/// the vision model, and returns them. The image bytes exist only for the
/// lifetime of this request handler — see ocr.rs for the privacy rationale.
async fn extract_handler(mut multipart: Multipart) -> impl IntoResponse {
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => return error(StatusCode::BAD_REQUEST, "no file field found"),
            // Most often the upload exceeded MAX_UPLOAD_BYTES. Report it rather
            // than falling through to a misleading "no file field found".
            Err(e) => {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("could not read upload: {e}"),
                )
            }
        };

        // Skip plain text fields; only a field carrying a filename is an upload.
        if field.file_name().is_none() {
            continue;
        }
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("could not read file contents: {e}"),
                )
            }
        };

        return match ocr::extract_fields_from_document(&data, &content_type).await {
            Ok(fields) => (StatusCode::OK, Json(fields)).into_response(),
            Err(e) => error(StatusCode::BAD_GATEWAY, e),
        };
    }
}
