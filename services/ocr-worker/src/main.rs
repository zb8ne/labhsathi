mod consumer;
mod downsample;
mod extract;
mod offset_tracker;
mod redis_cache;

use redis::aio::ConnectionManager;
use std::sync::Arc;

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
    // GitHub issue #4 (product review): no Arc<Mutex<..>> wrapper -- see
    // consumer.rs's `run` doc comment.
    let redis: ConnectionManager = ConnectionManager::new(redis_client)
        .await
        .expect("failed to connect to redis -- check REDIS_URL");
    let http_client = Arc::new(consumer::build_http_client());
    let max_concurrent_extractions = consumer::max_concurrent_extractions_from_env();

    tracing::info!(
        group = %group_id,
        max_concurrent_extractions,
        "ocr-worker started, consuming document.jobs.submitted"
    );

    // GitHub issue #11 (product review): without this, SIGTERM (every
    // Railway redeploy / `kubectl rollout`) kills the process immediately,
    // mid-extraction -- the in-flight job's image is already GETDEL'd out
    // of Redis by then, so a retry can't recover it; it just times out
    // client-side. This lets `run`'s receive loop see the signal and stop
    // pulling *new* messages, while in-flight `tokio::spawn`ed jobs (bounded
    // by MAX_CONCURRENT_EXTRACTIONS, each with its own VISION_API_TIMEOUT)
    // get a chance to finish and publish a real terminal event instead of
    // silently vanishing. Kafka's at-least-once redelivery is still the
    // backstop for whatever's mid-flight at the platform's actual kill
    // grace period, same as before this change.
    let shutdown = shutdown_signal();
    tokio::select! {
        _ = consumer::run(kafka_consumer, kafka_producer, redis, http_client, max_concurrent_extractions) => {}
        _ = shutdown => {
            tracing::info!("shutdown signal received, stopping consumption of new jobs");
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
