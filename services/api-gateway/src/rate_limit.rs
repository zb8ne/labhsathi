use axum::extract::ConnectInfo;
use axum::http::Request;
use governor::middleware::NoOpMiddleware;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tower_governor::errors::GovernorError;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::KeyExtractor;
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

/// GitHub issue #3 (product review): `tower_governor`'s built-in
/// `SmartIpKeyExtractor` trusts the *first* entry in `X-Forwarded-For`,
/// which is exactly the entry a client controls -- a request can set
/// `X-Forwarded-For: 1.2.3.4` (or a fresh random value per request) and
/// get a brand new rate-limit bucket every time, making the limit a no-op
/// against the one thing it exists to guard: unmetered spend against the
/// vision API on a publicly reachable upload endpoint.
///
/// This app sits behind exactly one trusted hop (Railway's edge proxy), so
/// the value that hop itself appended is the *last* entry in the header,
/// not the first -- a well-behaved reverse proxy appends the peer address
/// it actually saw rather than replacing the header outright, so the
/// rightmost entry is the one a client cannot forge (anything they put
/// there themselves ends up to its *left*, not in that position). Falls
/// back to `X-Real-IP`, then the raw connection peer (curl direct to the
/// port, no proxy in front -- local dev, `docker compose`, `kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustedProxyIpKeyExtractor;

// `KeyExtractor::name`/`key_name` are only part of the trait when
// tower_governor's own "tracing" feature is enabled (see its Cargo.toml) --
// this workspace doesn't enable it (default features are just ["axum"]),
// so implementing them here would be an error, not just dead code.
impl KeyExtractor for TrustedProxyIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        rightmost_forwarded_for(req)
            .or_else(|| x_real_ip(req))
            .or_else(|| connect_info(req))
            .ok_or(GovernorError::UnableToExtractKey)
    }
}

fn rightmost_forwarded_for<T>(req: &Request<T>) -> Option<IpAddr> {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.split(',').rev().find_map(|part| part.trim().parse::<IpAddr>().ok()))
}

fn x_real_ip<T>(req: &Request<T>) -> Option<IpAddr> {
    req.headers()
        .get("x-real-ip")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
}

fn connect_info<T>(req: &Request<T>) -> Option<IpAddr> {
    req.extensions().get::<ConnectInfo<SocketAddr>>().map(|ci| ci.0.ip())
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
pub fn upload_rate_limit_layer(config: &UploadRateLimitConfig) -> GovernorLayer<TrustedProxyIpKeyExtractor, NoOpMiddleware> {
    let governor_config: GovernorConfig<TrustedProxyIpKeyExtractor, NoOpMiddleware> = GovernorConfigBuilder::default()
        .key_extractor(TrustedProxyIpKeyExtractor)
        .burst_size(config.burst_size)
        .period(Duration::from_secs(config.period_secs))
        .finish()
        .expect("static governor config is always valid");

    GovernorLayer {
        config: Arc::new(governor_config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn request_with_xff(value: &str) -> Request<()> {
        let mut req = Request::new(());
        req.headers_mut().insert("x-forwarded-for", HeaderValue::from_str(value).unwrap());
        req
    }

    #[test]
    fn trusts_the_rightmost_forwarded_for_entry_not_the_leftmost() {
        // The bug this fixes: a client sends its own X-Forwarded-For with
        // an attacker-chosen first value; Railway's edge proxy appends the
        // real peer IP it saw as the *last* entry. The old SmartIpKeyExtractor
        // took the first (attacker-controlled) entry -- this must take the last.
        let req = request_with_xff("9.9.9.9, 203.0.113.7");
        let key = TrustedProxyIpKeyExtractor.extract(&req).expect("should extract a key");
        assert_eq!(key, "203.0.113.7".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn a_single_forged_entry_with_no_real_proxy_hop_still_extracts_something() {
        // If there's truly only one entry, there's nothing to distinguish
        // "client-supplied" from "proxy-supplied" -- this is the honest
        // limit of trusting exactly one hop, not a claim this defeats a
        // client that controls the network path entirely.
        let req = request_with_xff("1.2.3.4");
        let key = TrustedProxyIpKeyExtractor.extract(&req).expect("should extract a key");
        assert_eq!(key, "1.2.3.4".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn different_client_supplied_prefixes_with_the_same_real_ip_collide_on_the_same_key() {
        // The actual fix, proven directly: two requests spoofing different
        // fake first hops but arriving through the same real proxy (same
        // rightmost IP) must land on the *same* rate-limit bucket -- if
        // they didn't, the bypass would still work.
        let a = TrustedProxyIpKeyExtractor
            .extract(&request_with_xff("1.1.1.1, 203.0.113.7"))
            .unwrap();
        let b = TrustedProxyIpKeyExtractor
            .extract(&request_with_xff("random-garbage-not-even-an-ip, 203.0.113.7"))
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn falls_back_to_x_real_ip_when_no_forwarded_for_present() {
        let mut req = Request::new(());
        req.headers_mut().insert("x-real-ip", HeaderValue::from_static("198.51.100.23"));
        let key = TrustedProxyIpKeyExtractor.extract(&req).expect("should extract a key");
        assert_eq!(key, "198.51.100.23".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn falls_back_to_connect_info_when_no_headers_present() {
        let mut req = Request::new(());
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 54321))));
        let key = TrustedProxyIpKeyExtractor.extract(&req).expect("should extract a key");
        assert_eq!(key, "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn no_ip_signal_at_all_is_a_hard_error_not_a_shared_fallback_key() {
        let req = Request::new(());
        assert!(TrustedProxyIpKeyExtractor.extract(&req).is_err());
    }
}
