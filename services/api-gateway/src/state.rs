use rdkafka::producer::FutureProducer;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// `ConnectionManager` is internally shareable, but its methods take `&mut
/// self` -- wrap it in a Mutex so it can live behind Axum's `State`
/// extractor (which only gives out `&AppState`) without cloning a
/// connection per request.
#[derive(Clone)]
pub struct AppState {
    pub kafka_producer: Arc<FutureProducer>,
    pub redis: Arc<Mutex<ConnectionManager>>,
    /// Reused across requests -- reqwest::Client holds its own connection
    /// pool internally, building a fresh one per request would throw that
    /// away and pay a new TCP/TLS handshake to catalog-service every time.
    pub http_client: reqwest::Client,
    pub catalog_service_url: String,
}
