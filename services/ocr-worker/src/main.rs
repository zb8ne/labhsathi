mod consumer;
mod downsample;
mod extract;
mod offset_tracker;
mod redis_cache;

use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        tracing::warn!(
            "ANTHROPIC_API_KEY is not set -- this worker will publish a Failed event for \
             every job until it's provided (via Doppler or the environment)"
        );
    }

    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let group_id =
        std::env::var("KAFKA_CONSUMER_GROUP").unwrap_or_else(|_| "ocr-worker-group".to_string());

    let kafka_consumer = Arc::new(consumer::build_consumer(&kafka_brokers, &group_id));
    let kafka_producer = Arc::new(consumer::build_producer(&kafka_brokers));

    let redis_client = redis::Client::open(redis_url).expect("invalid REDIS_URL");
    let redis_conn = ConnectionManager::new(redis_client)
        .await
        .expect("failed to connect to redis -- check REDIS_URL");
    let redis = Arc::new(Mutex::new(redis_conn));
    let http_client = Arc::new(consumer::build_http_client());
    let max_concurrent_extractions = consumer::max_concurrent_extractions_from_env();

    tracing::info!(
        group = %group_id,
        max_concurrent_extractions,
        "ocr-worker started, consuming document.jobs.submitted"
    );
    consumer::run(kafka_consumer, kafka_producer, redis, http_client, max_concurrent_extractions).await;
}
