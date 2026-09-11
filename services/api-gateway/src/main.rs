mod kafka;
mod rate_limit;
mod redis_cache;
mod routes;
mod state;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use labhsathi_core::media::MAX_IMAGE_BYTES;
use redis::aio::ConnectionManager;
use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        tracing::warn!(
            "ANTHROPIC_API_KEY is not set in this service -- fine, ocr-worker is the one \
             that needs it. api-gateway never calls the vision API directly."
        );
    }

    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let kafka_producer = Arc::new(kafka::build_producer(&kafka_brokers));

    let redis_client = redis::Client::open(redis_url).expect("invalid REDIS_URL");
    let redis_conn = ConnectionManager::new(redis_client)
        .await
        .expect("failed to connect to redis -- check REDIS_URL");

    let app_state = AppState {
        kafka_producer,
        redis: Arc::new(Mutex::new(redis_conn)),
    };

    let upload_routes = Router::new()
        .route("/api/documents", post(routes::documents_upload_handler))
        .layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES))
        .layer(rate_limit::upload_rate_limit_layer());

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/match", post(routes::match_handler))
        .route("/api/documents/:job_id", get(routes::documents_status_handler))
        .merge(upload_routes)
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("api-gateway listening on http://{addr}");
    // SmartIpKeyExtractor (rate_limit.rs) falls back to the raw connection's
    // peer address when there's no X-Forwarded-For/X-Real-IP header (e.g.
    // curl direct to this port, no proxy in front) -- that fallback only
    // works if ConnectInfo is actually in the request extensions, which
    // requires opting in here.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn health() -> &'static str {
    "ok"
}
