use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use labhsathi_core::schemes::{seed_facts, SchemeFacts};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("failed to connect to postgres");

    // No migration framework for one table -- CREATE TABLE IF NOT EXISTS is
    // idempotent and this schema has had zero changes since it was written.
    // JSONB-per-row (keyed by id) rather than a typed column per field:
    // SchemeFacts already has a stable Serialize/Deserialize shape shared
    // with api-gateway over the wire, so storing it as one JSON blob means
    // schema changes are a Rust struct edit, not a migration -- the actual
    // structured-data win here is that it's queryable Postgres (`data->>'category_domain'`
    // works fine) with real backups, not a file no service owns.
    create_table_if_missing(&pool).await.expect("failed to create schemes table");

    sync_catalog(&pool).await;

    let state = AppState { pool };

    let app = Router::new()
        .route("/health", get(health))
        .route("/schemes", get(list_schemes))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8090".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("catalog-service listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

// No migration framework for one table -- CREATE TABLE IF NOT EXISTS is
// idempotent and this schema has had zero changes since it was written.
// JSONB-per-row (keyed by id) rather than a typed column per field:
// SchemeFacts already has a stable Serialize/Deserialize shape shared with
// api-gateway over the wire, so storing it as one JSON blob means schema
// changes are a Rust struct edit, not a migration -- the actual
// structured-data win here is that it's queryable Postgres
// (`data->>'category_domain'` works fine) with real backups, not a file no
// service owns.
async fn create_table_if_missing(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS schemes (id TEXT PRIMARY KEY, data JSONB NOT NULL)")
        .execute(pool)
        .await?;
    Ok(())
}

async fn fetch_all_schemes(pool: &PgPool) -> Result<Vec<SchemeFacts>, String> {
    let rows = sqlx::query_scalar::<_, serde_json::Value>("SELECT data FROM schemes ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("failed to query schemes: {e}"))?;

    rows.into_iter()
        .map(serde_json::from_value)
        .collect::<Result<Vec<SchemeFacts>, _>>()
        .map_err(|e| format!("a stored scheme row didn't deserialize as SchemeFacts: {e}"))
}

/// The only route that matters: api-gateway calls this once per match
/// request to get the current catalog, then runs labhsathi-core's own
/// matching logic against it -- this service owns the data, not the
/// eligibility rules.
async fn list_schemes(State(state): State<AppState>) -> Result<Json<Vec<SchemeFacts>>, StatusCode> {
    fetch_all_schemes(&state.pool).await.map(Json).map_err(|e| {
        tracing::error!(error = %e, "GET /schemes failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// Runs on every boot, against a table that may already hold rows.
/// labhsathi-core's seed_facts() is the same reviewed data/schemes.json
/// this repo has always shipped -- Postgres is the runtime store now, but
/// the dataset itself is still the hand-curated one, not pulled from
/// anywhere live.
///
/// Deliberately not gated on "table is empty": that would mean the only
/// way to get a newly-added embedded scheme into a database that's
/// already been seeded once is a manual INSERT against production. Instead
/// this runs the same upsert-by-id loop every boot -- `ON CONFLICT (id) DO
/// UPDATE` overwrites the stored row with the embedded catalog's current
/// version every time (a real insert for anything new, a real update for
/// anything edited). This *is* the source of truth on every deploy; a
/// manual edit made directly against production Postgres (outside this
/// repo) would be silently reverted on the next boot -- there is no
/// supported "edit prod, not the repo" path, by design. Safe for every
/// replica to run this at once: two concurrent UPSERTs on the same id are
/// just two writes to the same row, not a race that can corrupt anything.
async fn sync_catalog(pool: &PgPool) {
    let facts = seed_facts();
    tracing::info!(count = facts.len(), "syncing schemes table against the embedded catalog");

    for f in &facts {
        let data = serde_json::to_value(f).expect("SchemeFacts must serialize");
        sqlx::query(
            "INSERT INTO schemes (id, data) VALUES ($1, $2) \
             ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data",
        )
        .bind(&f.id)
        .bind(&data)
        .execute(pool)
        .await
        .expect("failed to sync a scheme row");
    }
}

// These need a live Postgres and are excluded from the default `cargo test`
// run, same convention as services/api-gateway/src/redis_cache.rs's Redis
// tests: `cargo test -p catalog-service -- --ignored --test-threads=1`
// against a real DATABASE_URL (the standalone `docker run` in .env.example
// is easiest -- Compose's own postgres service isn't published to the
// host). --test-threads=1 matters here specifically: these tests all call
// CREATE TABLE IF NOT EXISTS against the same fresh database, and Postgres
// isn't safe against two concurrent callers racing that on a table that
// doesn't exist yet (confirmed -- running them in parallel intermittently
// throws a duplicate-key error on pg_catalog.pg_type). Not a production
// concern -- catalog-service only ever calls this once, at its own boot.
#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://labhsathi:labhsathi_dev_only@localhost:5432/labhsathi_catalog".to_string()
        });
        PgPool::connect(&url)
            .await
            .expect("could not connect to postgres -- is it running? see the #[ignore] note above this module")
    }

    #[tokio::test]
    #[ignore]
    async fn syncing_an_empty_table_populates_the_full_catalog() {
        let pool = test_pool().await;
        create_table_if_missing(&pool).await.expect("create table");
        sqlx::query("DELETE FROM schemes").execute(&pool).await.expect("clear table for test isolation");

        sync_catalog(&pool).await;

        let stored = fetch_all_schemes(&pool).await.expect("fetch schemes");
        let expected = seed_facts();
        assert_eq!(stored.len(), expected.len());

        let pm_kisan = stored.iter().find(|s| s.id == "pm-kisan").expect("pm-kisan present after syncing");
        assert_eq!(pm_kisan.source_url.as_deref(), Some("https://pmkisan.gov.in"));
    }

    #[tokio::test]
    #[ignore]
    async fn syncing_twice_against_an_already_populated_table_does_not_duplicate_rows() {
        let pool = test_pool().await;
        create_table_if_missing(&pool).await.expect("create table");
        sqlx::query("DELETE FROM schemes").execute(&pool).await.expect("clear table for test isolation");

        sync_catalog(&pool).await;
        let first_pass = fetch_all_schemes(&pool).await.expect("fetch schemes");

        // A second call must not duplicate rows or error -- ON CONFLICT DO
        // UPDATE is what makes it safe for every catalog-service replica to
        // run this same sync at boot without racing each other into a
        // broken state, and safe to run again on every future redeploy.
        // Row *count* staying stable is what this test checks; row
        // *content* actually changing on an edit is covered separately by
        // `syncing_overwrites_a_row_whose_embedded_data_changed` below --
        // this repo previously shipped `ON CONFLICT DO NOTHING`, which kept
        // row count stable too, while silently never applying an edit to
        // an existing scheme. Don't let a future change regress back to
        // that by only checking length here again.
        sync_catalog(&pool).await;
        let second_pass = fetch_all_schemes(&pool).await.expect("fetch schemes");

        assert_eq!(first_pass.len(), second_pass.len());
    }

    #[tokio::test]
    #[ignore]
    async fn syncing_overwrites_a_row_whose_embedded_data_changed() {
        // The actual bug this fixes: production Postgres previously never
        // received an edit to an existing scheme once that row's id had
        // been inserted once (ON CONFLICT DO NOTHING). Simulate exactly
        // that drift -- seed a row with stale data, then confirm a normal
        // sync_catalog() call (the same one that runs on every boot)
        // overwrites it with the current embedded catalog's version.
        let pool = test_pool().await;
        create_table_if_missing(&pool).await.expect("create table");
        sqlx::query("DELETE FROM schemes").execute(&pool).await.expect("clear table for test isolation");

        let mut stale = seed_facts().into_iter().find(|f| f.id == "pm-kisan").expect("pm-kisan in embedded catalog");
        stale.official_note = "THIS IS DELIBERATELY STALE TEST DATA".to_string();
        let stale_json = serde_json::to_value(&stale).expect("serialize");
        sqlx::query("INSERT INTO schemes (id, data) VALUES ($1, $2)")
            .bind(&stale.id)
            .bind(&stale_json)
            .execute(&pool)
            .await
            .expect("seed a deliberately stale row by hand");

        sync_catalog(&pool).await;

        let stored = fetch_all_schemes(&pool).await.expect("fetch schemes");
        let pm_kisan = stored.iter().find(|s| s.id == "pm-kisan").expect("pm-kisan present after syncing");
        let current = seed_facts().into_iter().find(|f| f.id == "pm-kisan").expect("pm-kisan in embedded catalog");
        assert_eq!(
            pm_kisan.official_note, current.official_note,
            "sync_catalog must overwrite a row that drifted from the embedded catalog, not leave it stale"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn syncing_adds_new_rows_to_a_table_that_already_has_some() {
        let pool = test_pool().await;
        create_table_if_missing(&pool).await.expect("create table");
        sqlx::query("DELETE FROM schemes").execute(&pool).await.expect("clear table for test isolation");

        // Simulate "already deployed with an older, smaller catalog": seed
        // just one real row by hand, then run the normal sync and confirm
        // it fills in the rest without touching the pre-existing one.
        let pm_kisan = seed_facts().into_iter().find(|f| f.id == "pm-kisan").expect("pm-kisan in embedded catalog");
        let data = serde_json::to_value(&pm_kisan).expect("serialize");
        sqlx::query("INSERT INTO schemes (id, data) VALUES ($1, $2)")
            .bind(&pm_kisan.id)
            .bind(&data)
            .execute(&pool)
            .await
            .expect("seed one row by hand");

        sync_catalog(&pool).await;

        let stored = fetch_all_schemes(&pool).await.expect("fetch schemes");
        let expected = seed_facts();
        assert_eq!(stored.len(), expected.len(), "sync should have added every other embedded scheme");
    }
}
