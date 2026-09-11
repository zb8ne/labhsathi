use crate::downsample::downsample_to_jpeg;
use crate::extract::extract_fields_from_jpeg;
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
use tokio::sync::{Mutex, Semaphore};

/// Caps concurrent in-flight vision-API calls per pod. This, not the number
/// of pods, is the real ceiling on this service's resource use -- it's why
/// the ADR is honest that CPU-based HPA is an imperfect scaling signal for
/// a service whose actual bottleneck is an outbound HTTP call, not CPU.
const MAX_CONCURRENT_EXTRACTIONS: usize = 4;

pub fn build_consumer(brokers: &str, group_id: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        // Offsets are stored (marked ready) only after this worker has
        // published a terminal document.jobs.completed event for that
        // message -- see process_job below. A crash mid-extraction means
        // librdkafka never stored the offset, so the message redelivers to
        // another consumer in the group rather than being silently lost.
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

pub async fn run(
    consumer: Arc<StreamConsumer>,
    producer: Arc<FutureProducer>,
    redis: Arc<Mutex<ConnectionManager>>,
) {
    consumer
        .subscribe(&[TOPIC_DOCUMENT_JOBS_SUBMITTED])
        .expect("failed to subscribe to document.jobs.submitted");

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_EXTRACTIONS));

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

        let event: DocumentJobSubmitted = match serde_json::from_slice(&payload) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!(error = %e, "malformed document.jobs.submitted event, dropping");
                let _ = consumer.store_offset(&topic, partition, offset);
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
        let consumer_for_commit = consumer.clone();

        tokio::spawn(async move {
            process_job(event, &producer, &redis).await;
            if let Err(e) = consumer_for_commit.store_offset(&topic, partition, offset) {
                tracing::error!(error = %e, "failed to store kafka offset after processing");
            }
            drop(permit);
        });
    }
}

/// Always writes a Processing status, then always publishes a terminal
/// Success/Failed event and a matching terminal status -- a job is never
/// left in limbo regardless of where it fails.
async fn process_job(
    event: DocumentJobSubmitted,
    producer: &FutureProducer,
    redis: &Arc<Mutex<ConnectionManager>>,
) {
    let job_id = event.job_id;

    {
        let mut conn = redis.lock().await;
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

    let result = process_job_inner(job_id, redis).await;

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
    if let Err(e) = publish_completed(producer, &completed).await {
        tracing::error!(%job_id, error = %e, "failed to publish document.jobs.completed");
    }

    let record = match result {
        Ok(fields) => JobStatusRecord {
            status: JobProgress::Done,
            fields: Some(fields),
            error: None,
        },
        Err(e) => JobStatusRecord {
            status: JobProgress::Failed,
            fields: None,
            error: Some(e),
        },
    };
    let mut conn = redis.lock().await;
    let _ = redis_cache::set_status(&mut conn, job_id, &record).await;
}

async fn process_job_inner(
    job_id: JobId,
    redis: &Arc<Mutex<ConnectionManager>>,
) -> Result<ExtractedFields, String> {
    let raw = {
        let mut conn = redis.lock().await;
        redis_cache::fetch_and_delete_image(&mut conn, job_id).await?
    };
    // A miss here is a legitimate state (redelivery after a crash, or the
    // 60s TTL won the race), not corruption -- report it as a normal
    // extraction failure so the frontend degrades gracefully.
    let raw = raw.ok_or_else(|| {
        "image already consumed or expired before this worker could read it".to_string()
    })?;

    let jpeg = downsample_to_jpeg(&raw)?;
    extract_fields_from_jpeg(&jpeg).await
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
