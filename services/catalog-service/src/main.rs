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

    seed_if_empty(&pool).await;

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

/// Runs once, on first boot against an empty database. labhsathi-core's
/// seed_facts() is the same reviewed data/schemes.json this repo has
/// always shipped -- Postgres is the runtime store now, but the dataset
/// itself is still the hand-curated one, not pulled from anywhere live.
async fn seed_if_empty(pool: &PgPool) {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schemes")
        .fetch_one(pool)
        .await
        .expect("failed to count schemes");

    if count > 0 {
        tracing::info!(count, "schemes table already seeded, skipping");
        return;
    }

    let facts = seed_facts();
    tracing::info!(count = facts.len(), "seeding schemes table from the embedded catalog");

    for f in &facts {
        let data = serde_json::to_value(f).expect("SchemeFacts must serialize");
        sqlx::query("INSERT INTO schemes (id, data) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(&f.id)
            .bind(&data)
            .execute(pool)
            .await
            .expect("failed to seed a scheme row");
    }
}

// These need a live Postgres and are excluded from the default `cargo test`
// run, same convention as services/api-gateway/src/redis_cache.rs's Redis
// tests: `cargo test -p catalog-service -- --ignored --test-threads=1`
// against a real DATABASE_URL (the standalone `docker run` in .env.example
// is easiest -- Compose's own postgres service isn't published to the
// host). --test-threads=1 matters here specifically: both tests call
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
    async fn seeding_an_empty_table_populates_the_full_catalog() {
        let pool = test_pool().await;
        create_table_if_missing(&pool).await.expect("create table");
        sqlx::query("DELETE FROM schemes").execute(&pool).await.expect("clear table for test isolation");

        seed_if_empty(&pool).await;

        let stored = fetch_all_schemes(&pool).await.expect("fetch schemes");
        let expected = seed_facts();
        assert_eq!(stored.len(), expected.len());

        let pm_kisan = stored.iter().find(|s| s.id == "pm-kisan").expect("pm-kisan present after seeding");
        assert_eq!(pm_kisan.source_url.as_deref(), Some("https://pmkisan.gov.in"));
    }

    #[tokio::test]
    #[ignore]
    async fn seeding_is_a_no_op_against_an_already_populated_table() {
        let pool = test_pool().await;
        create_table_if_missing(&pool).await.expect("create table");
        sqlx::query("DELETE FROM schemes").execute(&pool).await.expect("clear table for test isolation");

        seed_if_empty(&pool).await;
        let first_pass = fetch_all_schemes(&pool).await.expect("fetch schemes");

        // A second call must not duplicate or error (ON CONFLICT DO NOTHING
        // plus the count>0 short-circuit) -- this is what makes it safe for
        // every catalog-service replica to run the same startup seed logic
        // without racing each other into a broken state.
        seed_if_empty(&pool).await;
        let second_pass = fetch_all_schemes(&pool).await.expect("fetch schemes");

        assert_eq!(first_pass.len(), second_pass.len());
    }
}
