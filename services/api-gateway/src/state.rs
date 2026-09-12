use rdkafka::producer::FutureProducer;
use redis::aio::ConnectionManager;
use std::sync::Arc;

/// GitHub issue #4 (product review): `ConnectionManager` methods take `&mut
/// self` only because of the `AsyncCommands` trait's generic bounds, not
/// because the connection needs exclusive access -- it's built specifically
/// to be cloned freely: internally it holds its connection behind its own
/// shared, reconnecting handle, and a clone is a cheap handle copy, not a
/// new TCP connection. Wrapping it in a `tokio::sync::Mutex` (the previous
/// version of this struct) serialized *every* Redis operation across *every*
/// concurrent request through one lock -- including the upload handler,
/// which held that lock for the full duration of the Kafka publish (up to
/// its 5s timeout), so a single slow Kafka publish stalled every other
/// request's status poll behind it. `ConnectionManager` is cloned directly
/// into each request via `AppState`'s own `Clone` instead.
#[derive(Clone)]
pub struct AppState {
    pub kafka_producer: Arc<FutureProducer>,
    pub redis: ConnectionManager,
    /// Reused across requests -- reqwest::Client holds its own connection
    /// pool internally, building a fresh one per request would throw that
    /// away and pay a new TCP/TLS handshake to catalog-service every time.
    pub http_client: reqwest::Client,
    pub catalog_service_url: String,
    /// In-memory cache of the last successful catalog-service response,
    /// with the instant it was fetched -- see GitHub issue #7 and
    /// catalog_client.rs. Behind an `Arc<RwLock<..>>` since `AppState` is
    /// cloned per request but the cache itself must be shared.
    pub catalog_cache: Arc<tokio::sync::RwLock<Option<crate::catalog_client::CachedCatalog>>>,
}
