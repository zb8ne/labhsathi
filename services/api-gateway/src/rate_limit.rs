use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;

const DEFAULT_BURST_SIZE: u32 = 5;
const DEFAULT_PERIOD_SECS: u64 = 720;

/// GitHub issue #5: these were hardcoded, so the kind demo's HPA-scaling
/// load test had no way to generate enough traffic to matter without
/// rebuilding the binary. Read from env vars (Helm-settable via
/// apiGateway.uploadRateLimit.* in values.yaml), same defaults as before
/// when unset -- see infra/k8s/labhsathi/README.md for the load command
/// this makes possible.
pub struct UploadRateLimitConfig {
    pub burst_size: u32,
    pub period_secs: u64,
}

impl UploadRateLimitConfig {
    pub fn from_env() -> Self {
        let burst_size = std::env::var("UPLOAD_RATE_LIMIT_BURST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_BURST_SIZE);
        let period_secs = std::env::var("UPLOAD_RATE_LIMIT_PERIOD_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_PERIOD_SECS);
        Self { burst_size, period_secs }
    }
}

/// Per-IP limit on the document-upload route only -- this is the guard
/// against unmetered spend against the vision API on a publicly reachable
/// endpoint, not a general API rate limit. `/api/match` (pure Rust, no LLM
/// call) is deliberately not behind this layer.
///
/// GCRA doesn't have a native "N per hour" knob -- it's expressed as a
/// burst size plus a steady replenishment period. Default ~5/hour: burst
/// of 5 (an initial cluster of uploads succeeds immediately), then one
/// more token every 720s (3600s / 5). A client that uploads in a tight
/// loop gets `burst_size` through, then is throttled to one every
/// `period_secs`.
///
/// `SmartIpKeyExtractor` reads X-Forwarded-For / X-Real-IP first, falling
/// back to the raw peer address -- required behind Railway's edge proxy,
/// where the raw TCP peer is the proxy itself, not the client.
pub fn upload_rate_limit_layer(config: &UploadRateLimitConfig) -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware> {
    let governor_config: GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware> = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .burst_size(config.burst_size)
        .period(Duration::from_secs(config.period_secs))
        .finish()
        .expect("static governor config is always valid");

    GovernorLayer {
        config: Arc::new(governor_config),
    }
}
