use labhsathi_core::events::{DocumentJobSubmitted, TOPIC_DOCUMENT_JOBS_SUBMITTED};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration;

pub fn build_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("failed to build Kafka producer -- check KAFKA_BROKERS")
}

/// Publishes the submitted-job event. Key = job_id, so all events for one
/// job land on the same partition and stay ordered.
pub async fn publish_job_submitted(
    producer: &FutureProducer,
    event: &DocumentJobSubmitted,
) -> Result<(), String> {
    let key = event.job_id.to_string();
    let payload = serde_json::to_vec(event).map_err(|e| format!("serialize event: {e}"))?;

    producer
        .send(
            FutureRecord::to(TOPIC_DOCUMENT_JOBS_SUBMITTED)
                .key(&key)
                .payload(&payload),
            Timeout::After(Duration::from_secs(5)),
        )
        .await
        .map_err(|(e, _)| format!("kafka publish failed: {e}"))?;

    Ok(())
}
