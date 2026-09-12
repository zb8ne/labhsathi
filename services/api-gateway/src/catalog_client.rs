use labhsathi_core::schemes::SchemeFacts;

/// api-gateway holds no scheme data of its own -- every `/api/match` call
/// fetches the current catalog fresh from catalog-service (43 rows over
/// HTTP is sub-millisecond work; no caching, so a catalog edit in Postgres
/// is visible on the very next request rather than after a redeploy).
pub async fn fetch_schemes(client: &reqwest::Client, base_url: &str) -> Result<Vec<SchemeFacts>, String> {
    let url = format!("{base_url}/schemes");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("catalog-service unreachable: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("catalog-service returned {}", resp.status()));
    }

    resp.json::<Vec<SchemeFacts>>()
        .await
        .map_err(|e| format!("catalog-service response didn't parse as the expected shape: {e}"))
}
