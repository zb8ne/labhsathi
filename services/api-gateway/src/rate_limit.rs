use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;

/// Per-IP limit on the document-upload route only -- this is the guard
/// against unmetered spend against the vision API on a publicly reachable
/// endpoint, not a general API rate limit. `/api/match` (pure Rust, no LLM
/// call) is deliberately not behind this layer.
///
/// GCRA doesn't have a native "N per hour" knob -- it's expressed as a
/// burst size plus a steady replenishment period. ~5/hour: burst of 5
/// (an initial cluster of uploads succeeds immediately), then one more
/// token every 720s (3600s / 5). A client that uploads in a tight loop
/// gets 5 through, then is throttled to roughly one every 12 minutes.
///
/// `SmartIpKeyExtractor` reads X-Forwarded-For / X-Real-IP first, falling
/// back to the raw peer address -- required behind Railway's edge proxy,
/// where the raw TCP peer is the proxy itself, not the client.
pub fn upload_rate_limit_layer() -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware> {
    let config: GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware> = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .burst_size(5)
        .period(Duration::from_secs(720))
        .finish()
        .expect("static governor config is always valid");

    GovernorLayer {
        config: Arc::new(config),
    }
}
