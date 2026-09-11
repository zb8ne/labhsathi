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
}
