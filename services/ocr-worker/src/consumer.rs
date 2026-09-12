use crate::downsample::downsample_to_jpeg;
use crate::extract::extract_fields_from_jpeg;
use crate::offset_tracker::OffsetTracker;
use crate::redis_cache;
use labhsathi_core::events::{
    DocumentJobCompleted, DocumentJobSubmitted, ExtractedFields, JobId, JobStatus,
    TOPIC_DOCUMENT_JOBS_COMPLETED, TOPIC_DOCUMENT_JOBS_SUBMITTED,
};
use labhsathi_core::status::{JobProgress, JobStatusRecord};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Caps concurrent in-flight vision-API calls per pod. This, not the number
/// of pods, is the real ceiling on this service's resource use -- it's why
/// the ADR is honest that CPU-based HPA is an imperfect scaling signal for
/// a service whose actual bottleneck is an outbound HTTP call, not CPU.
///
/// GitHub issue #5: was a hardcoded const, so tuning the kind HPA demo's
/// load meant rebuilding the image. Now `MAX_CONCURRENT_EXTRACTIONS` (env,
/// Helm-settable via ocrWorker.maxConcurrentExtractions in values.yaml).
const DEFAULT_MAX_CONCURRENT_EXTRACTIONS: usize = 4;

pub fn max_concurrent_extractions_from_env() -> usize {
    std::env::var("MAX_CONCURRENT_EXTRACTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_CONCURRENT_EXTRACTIONS)
}

/// Generous but bounded -- GitHub issue #7: with no timeout at all, a
/// single hung call to the vision API holds its semaphore permit forever,
/// and enough of those exhaust every permit and stop the worker cold.
const VISION_API_TIMEOUT: Duration = Duration::from_secs(45);

pub fn build_consumer(brokers: &str, group_id: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        // Offsets are stored (marked ready) only after this worker has
        // published a terminal document.jobs.completed event for that
        // message AND the offset tracker confirms it closes a contiguous
        // run (see offset_tracker.rs) -- see process_job/run below. A
        // crash mid-extraction, or a completed-event publish that keeps
        // failing, means librdkafka never stores that offset, so the
        // message (and everything after it on the partition) redelivers.
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set("auto.commit.interval.ms", "1000")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("failed to build Kafka consumer -- check KAFKA_BROKERS")
}

pub fn build_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("failed to build Kafka producer -- check KAFKA_BROKERS")
}

/// One shared client for every vision-API call -- reqwest's client owns a
/// connection pool, and building a fresh one per request throws that away.
/// The timeout here is what makes GitHub issue #7's fix real.
pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(VISION_API_TIMEOUT)
        .build()
        .expect("failed to build reqwest client")
}

pub async fn run(
    consumer: Arc<StreamConsumer>,
    producer: Arc<FutureProducer>,
    // GitHub issue #4 (product review): `ConnectionManager` is `Clone` by
    // design specifically so it can be shared this way -- see the doc
    // comment on api-gateway/src/state.rs's `AppState::redis` for the full
    // reasoning. No `Arc<Mutex<..>>` wrapper needed or wanted.
    redis: ConnectionManager,
    http_client: Arc<reqwest::Client>,
    max_concurrent_extractions: usize,
) {
    consumer
        .subscribe(&[TOPIC_DOCUMENT_JOBS_SUBMITTED])
        .expect("failed to subscribe to document.jobs.submitted");

    let semaphore = Arc::new(Semaphore::new(max_concurrent_extractions));
    let offsets = Arc::new(OffsetTracker::new());

    loop {
        let msg = match consumer.recv().await {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!(error = %e, "kafka recv error, retrying");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        // Copy out everything we need as owned values before `msg` (which
        // borrows from the consumer) goes out of scope -- nothing borrowed
        // crosses the spawn boundary below.
        let payload = match msg.payload() {
            Some(p) => p.to_vec(),
            None => {
                tracing::warn!("empty message payload, skipping");
                continue;
            }
        };
        let topic = msg.topic().to_string();
        let partition = msg.partition();
        let offset = msg.offset();
        drop(msg);

        // Must happen here, synchronously, in receipt order -- see
        // offset_tracker.rs. Every offset that reaches `complete()` below
        // (including the malformed-message early-exit) must be registered
        // first, or `complete()` panics.
        offsets.register_received(partition, offset);

        let event: DocumentJobSubmitted = match serde_json::from_slice(&payload) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!(error = %e, "malformed document.jobs.submitted event, dropping");
                // A message that can never be processed (not a transient
                // failure) is still a terminal state for ordering purposes
                // -- feed it through the tracker like any other completion
                // so it doesn't permanently block everything after it.
                for ready in offsets.complete(partition, offset) {
                    let _ = consumer.store_offset(&topic, partition, ready);
                }
                continue;
            }
        };

        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore never closes");
        let producer = producer.clone();
        let redis = redis.clone();
        let http_client = http_client.clone();
        let consumer_for_commit = consumer.clone();
        let offsets = offsets.clone();

        tokio::spawn(async move {
            let published = process_job(event, &producer, &redis, &http_client).await;

            // Only offsets whose completed-event actually published are
            // fed into the tracker -- a publish failure must NOT be
            // treated as terminal, or this message (and its bytes, already
            // gone from Redis via GETDEL) would be marked done with no
            // record of it ever having succeeded or failed anywhere.
            if published {
                for ready in offsets.complete(partition, offset) {
                    if let Err(e) = consumer_for_commit.store_offset(&topic, partition, ready) {
                        tracing::error!(error = %e, offset = ready, "failed to store kafka offset after processing");
                    }
                }
            } else {
                tracing::error!(
                    %partition, %offset,
                    "document.jobs.completed publish failed -- this offset and everything \
                     after it on this partition will redeliver on next restart"
                );
            }
            drop(permit);
        });
    }
}

/// GitHub issue #13 (product review): `process_job_inner`'s error strings
/// include raw detail from the vision API (`extract.rs`'s `API error
/// ({status}): {msg}`, which can echo back auth/billing/quota text from
/// Anthropic) and from image decoding -- fine for a server log, not fine
/// served verbatim through api-gateway's public, unauthenticated `GET
/// /api/documents/{job_id}` status endpoint, which anyone holding a job id
/// can poll. This maps the handful of error shapes this worker actually
/// produces to a small set of messages safe to show a user; the raw detail
/// still reaches the logs via the `tracing::error!` call at the one place
/// it's swallowed, in `process_job` below.
fn sanitize_error_for_client(raw: &str) -> String {
    // The one error message this worker constructs specifically to be
    // read by a user (see process_job_inner) -- pass it through unchanged
    // rather than genericizing a message that was already safe and useful.
    if raw.contains("already consumed or expired") {
        return raw.to_string();
    }
    if raw.contains("timed out") {
        return "The document scan took too long. Please try again.".to_string();
    }
    "Could not read this document. Please try again or fill the form manually.".to_string()
}

/// Always writes a Processing status, then always attempts to publish a
/// terminal Success/Failed event and a matching terminal status -- a job
/// is never left in limbo regardless of where extraction itself fails.
/// Returns whether the completed-event publish succeeded, which is what
/// the caller uses to decide whether this offset is safe to commit.
async fn process_job(
    event: DocumentJobSubmitted,
    producer: &FutureProducer,
    redis: &ConnectionManager,
    http_client: &reqwest::Client,
) -> bool {
    let job_id = event.job_id;

    {
        // GitHub issue #4 (product review): cloned directly, same reasoning
        // as api-gateway/src/state.rs -- ConnectionManager is built to be
        // shared this way, and with MAX_CONCURRENT_EXTRACTIONS > 1 this
        // worker has multiple jobs genuinely running in parallel, which a
        // shared Mutex would have serialized through one lock for no reason.
        let mut conn = redis.clone();
        let _ = redis_cache::set_status(
            &mut conn,
            job_id,
            &JobStatusRecord {
                status: JobProgress::Processing,
                fields: None,
                error: None,
            },
        )
        .await;
    }

    let result = process_job_inner(job_id, redis, http_client).await;

    if let Err(e) = &result {
        // The one place the raw, potentially-sensitive error detail is
        // allowed to exist in full -- server logs, not the public status
        // endpoint. See sanitize_error_for_client above.
        tracing::error!(%job_id, error = %e, "document extraction failed");
    }

    let (status, fields) = match &result {
        Ok(fields) => (JobStatus::Success, Some(fields.clone())),
        Err(_) => (JobStatus::Failed, None),
    };

    let completed = DocumentJobCompleted {
        job_id,
        status,
        fields: fields.clone(),
        ts: now_millis(),
    };
    let publish_ok = match publish_completed(producer, &completed).await {
        Ok(()) => true,
        Err(e) => {
            tracing::error!(%job_id, error = %e, "failed to publish document.jobs.completed");
            false
        }
    };

    let record = match result {
        Ok(fields) => JobStatusRecord {
            status: JobProgress::Done,
            fields: Some(fields),
            error: None,
        },
        Err(e) => JobStatusRecord {
            status: JobProgress::Failed,
            fields: None,
            error: Some(sanitize_error_for_client(&e)),
        },
    };

    // GitHub issue #12 (product review): a redelivered message (the
    // consumer crashed after the completed-event published but before the
    // offset committed, or the offset publish itself failed and this job
    // legitimately reprocesses) can race a slow first attempt's Done write
    // here. Don't let a redelivery's Failed status clobber an already-Done
    // one -- the frontend has already shown the user their real result by
    // then, and flipping it to Failed under them is strictly worse than
    // leaving the correct terminal state alone.
    let mut conn = redis.clone();
    if record.status == JobProgress::Failed {
        if let Ok(Some(existing)) = redis_cache::get_status(&mut conn, job_id).await {
            if existing.status == JobProgress::Done {
                tracing::warn!(%job_id, "redelivery failed after an earlier attempt already succeeded -- keeping the Done status, not overwriting with Failed");
                return publish_ok;
            }
        }
    }
    let _ = redis_cache::set_status(&mut conn, job_id, &record).await;

    publish_ok
}

async fn process_job_inner(
    job_id: JobId,
    redis: &ConnectionManager,
    http_client: &reqwest::Client,
) -> Result<ExtractedFields, String> {
    let raw = {
        let mut conn = redis.clone();
        redis_cache::fetch_and_delete_image(&mut conn, job_id).await?
    };
    // A miss here is a legitimate state (redelivery after a crash, or the
    // 60s TTL won the race), not corruption -- report it as a normal
    // extraction failure so the frontend degrades gracefully.
    let raw = raw.ok_or_else(|| {
        "image already consumed or expired before this worker could read it".to_string()
    })?;

    let jpeg = downsample_to_jpeg(&raw)?;
    extract_fields_from_jpeg(http_client, &jpeg).await
}

async fn publish_completed(producer: &FutureProducer, event: &DocumentJobCompleted) -> Result<(), String> {
    let key = event.job_id.to_string();
    let payload = serde_json::to_vec(event).map_err(|e| format!("serialize event: {e}"))?;
    producer
        .send(
            FutureRecord::to(TOPIC_DOCUMENT_JOBS_COMPLETED)
                .key(&key)
                .payload(&payload),
            Timeout::After(Duration::from_secs(5)),
        )
        .await
        .map_err(|(e, _)| format!("kafka publish failed: {e}"))?;
    Ok(())
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
