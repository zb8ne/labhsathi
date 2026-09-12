mod catalog_client;
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
use std::time::Duration;
use tokio::sync::RwLock;
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
    let catalog_service_url =
        std::env::var("CATALOG_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

    let kafka_producer = Arc::new(kafka::build_producer(&kafka_brokers));

    let redis_client = redis::Client::open(redis_url).expect("invalid REDIS_URL");
    let redis_conn = ConnectionManager::new(redis_client)
        .await
        .expect("failed to connect to redis -- check REDIS_URL");

    let app_state = AppState {
        kafka_producer,
        redis: redis_conn,
        // A default timeout here is defense in depth -- the actual guard
        // against a hung catalog-service call is the per-request
        // `.timeout()` in catalog_client.rs, which is what really matters
        // since it can't be silently forgotten on a future call site that
        // reuses this same client for something else.
        http_client: reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client"),
        catalog_service_url,
        catalog_cache: Arc::new(RwLock::new(None)),
    };

    let rate_limit_config = rate_limit::UploadRateLimitConfig::from_env();
    tracing::info!(
        burst_size = rate_limit_config.burst_size,
        period_secs = rate_limit_config.period_secs,
        "upload rate limit configured"
    );
    let upload_routes = Router::new()
        .route("/api/documents", post(routes::documents_upload_handler))
        .layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES))
        .layer(rate_limit::upload_rate_limit_layer(&rate_limit_config));

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
    // TrustedProxyIpKeyExtractor (rate_limit.rs) falls back to the raw
    // connection's peer address when there's no X-Forwarded-For/X-Real-IP
    // header (e.g. curl direct to this port, no proxy in front) -- that
    // fallback only works if ConnectInfo is actually in the request
    // extensions, which requires opting in here.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn health() -> &'static str {
    "ok"
}

/// GitHub issue #11 (product review): without this, every Railway redeploy
/// (or `kubectl rollout`) sends SIGTERM and the process dies mid-request --
/// a scan already in flight, or a status poll landing right as the pod
/// exits. `axum::serve`'s graceful shutdown stops accepting *new*
/// connections on the signal but lets in-flight requests finish (bounded by
/// the platform's own kill grace period, typically several seconds), which
/// covers the common redeploy-during-a-quick-request case. It does not by
/// itself cover a scan whose extraction is still running in ocr-worker at
/// the moment api-gateway exits -- that's a separate service with its own
/// shutdown handling, see ocr-worker/src/main.rs.
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
        _ = ctrl_c => tracing::info!("received Ctrl+C, shutting down gracefully"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down gracefully"),
    }
}
