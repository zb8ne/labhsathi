use labhsathi_core::schemes::SchemeFacts;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// GitHub issue #7 (product review): this used to be a bare, uncached HTTP
/// call on every single `/api/match` request with no timeout on the client
/// (see main.rs's `reqwest::Client::new()`) -- a slow or hung
/// catalog-service made every match request hang right along with it, with
/// nothing to time it out. It also meant every match request paid a real
/// network round-trip plus parsing ~100 JSON rows even though the catalog
/// changes on the order of "someone edits a scheme," not "someone submits
/// the form." Measured live before this fix: 1.3-1.4s per `/api/match`
/// call, almost entirely this round-trip.
///
/// A short cache closes both gaps without reintroducing the staleness this
/// module's original doc comment was written to avoid: a catalog edit is
/// visible within `CACHE_TTL`, not "after a redeploy" -- 30s felt like the
/// right trade for a hackathon-scope prototype (fast enough that an editor
/// testing a change doesn't wonder if it took), not a claim that this is
/// the right number for a system with real edit-then-verify SLAs.
const CACHE_TTL: Duration = Duration::from_secs(30);
const CATALOG_FETCH_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct CachedCatalog {
    pub fetched_at: Instant,
    pub facts: Vec<SchemeFacts>,
}

/// api-gateway holds no scheme data of its own -- the catalog always comes
/// from catalog-service, just no more often than once per `CACHE_TTL`. A
/// fetch failure with a still-fresh-enough cached value present falls back
/// to serving the stale cache rather than failing the request outright --
/// catalog-service being briefly unreachable shouldn't take `/api/match`
/// down with it when there's a perfectly good answer from 20 seconds ago.
pub async fn fetch_schemes(
    client: &reqwest::Client,
    base_url: &str,
    cache: &Arc<RwLock<Option<CachedCatalog>>>,
) -> Result<Vec<SchemeFacts>, String> {
    if let Some(cached) = cache.read().await.as_ref() {
        if cached.fetched_at.elapsed() < CACHE_TTL {
            return Ok(cached.facts.clone());
        }
    }

    match fetch_from_catalog_service(client, base_url).await {
        Ok(facts) => {
            let mut guard = cache.write().await;
            *guard = Some(CachedCatalog { fetched_at: Instant::now(), facts: facts.clone() });
            Ok(facts)
        }
        Err(e) => {
            // Serve a stale-but-present cache rather than failing outright
            // -- see the module doc above. A cold cache (first request
            // ever, or catalog-service has never once succeeded) still
            // propagates the real error, since there is nothing to fall
            // back to.
            if let Some(cached) = cache.read().await.as_ref() {
                tracing::warn!(error = %e, "catalog-service fetch failed, serving stale cached catalog");
                return Ok(cached.facts.clone());
            }
            Err(e)
        }
    }
}

async fn fetch_from_catalog_service(client: &reqwest::Client, base_url: &str) -> Result<Vec<SchemeFacts>, String> {
    let url = format!("{base_url}/schemes");
    let resp = client
        .get(&url)
        .timeout(CATALOG_FETCH_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "catalog-service request timed out".to_string()
            } else {
                format!("catalog-service unreachable: {e}")
            }
        })?;

    if !resp.status().is_success() {
        return Err(format!("catalog-service returned {}", resp.status()));
    }

    resp.json::<Vec<SchemeFacts>>()
        .await
        .map_err(|e| format!("catalog-service response didn't parse as the expected shape: {e}"))
}
