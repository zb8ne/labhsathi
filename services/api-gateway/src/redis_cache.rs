use labhsathi_core::events::JobId;
use labhsathi_core::status::JobStatusRecord;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

/// Raw image bytes, keyed by job id. 60s TTL, fetched exactly once by
/// ocr-worker via GETDEL -- this key is gone the moment it's read, and gone
/// on its own after 60s even if nothing ever reads it. This is the entire
/// surface where an uploaded document's bytes exist anywhere in the system.
fn image_key(job_id: JobId) -> String {
    format!("labhsathi:doc:{job_id}")
}

/// A small JSON status blob, keyed by job id. Separate key, separate
/// (longer) TTL from `image_key` -- this is the concrete answer to "Redis
/// is a cache, not a datastore": the read-model key holds a few bytes of
/// status for a few minutes, never image data, never indefinitely.
fn status_key(job_id: JobId) -> String {
    format!("labhsathi:job:{job_id}")
}

const IMAGE_TTL_SECS: u64 = 60;
const STATUS_TTL_SECS: u64 = 300;

pub async fn store_image(
    conn: &mut ConnectionManager,
    job_id: JobId,
    bytes: &[u8],
) -> Result<(), String> {
    // SET ... NX EX -- refuses to overwrite an existing key (job ids are
    // fresh UUIDs, so a collision would only mean something is badly wrong)
    // and sets the TTL atomically with the write.
    let set: Option<String> = redis::cmd("SET")
        .arg(image_key(job_id))
        .arg(bytes)
        .arg("EX")
        .arg(IMAGE_TTL_SECS)
        .arg("NX")
        .query_async(conn)
        .await
        .map_err(|e| format!("redis SET failed: {e}"))?;

    if set.is_none() {
        return Err(format!("job id collision writing image for {job_id}"));
    }
    Ok(())
}

/// Best-effort cleanup for the compensating-action path: if the Kafka
/// publish fails after the Redis write succeeded, delete the orphaned key
/// rather than leaving it to expire silently.
pub async fn delete_image(conn: &mut ConnectionManager, job_id: JobId) {
    let _: Result<i64, _> = conn.del(image_key(job_id)).await;
}

pub async fn set_status(
    conn: &mut ConnectionManager,
    job_id: JobId,
    record: &JobStatusRecord,
) -> Result<(), String> {
    let payload = serde_json::to_string(record).map_err(|e| format!("serialize status: {e}"))?;
    conn.set_ex::<_, _, ()>(status_key(job_id), payload, STATUS_TTL_SECS)
        .await
        .map_err(|e| format!("redis SETEX failed: {e}"))
}

pub async fn get_status(
    conn: &mut ConnectionManager,
    job_id: JobId,
) -> Result<Option<JobStatusRecord>, String> {
    let raw: Option<String> = conn
        .get(status_key(job_id))
        .await
        .map_err(|e| format!("redis GET failed: {e}"))?;

    match raw {
        None => Ok(None),
        Some(s) => serde_json::from_str(&s)
            .map(Some)
            .map_err(|e| format!("corrupt status record: {e}")),
    }
}

/// GitHub issue #4: covers the upload-compensation path (documents.rs
/// deletes the just-written image key if the Kafka publish fails) against
/// a real Redis rather than mocking the client -- there's a real Redis in
/// every environment this runs in (Compose/kind/Railway), and mocking
/// redis::aio::ConnectionManager realistically enough to trust the result
/// is more effort than just running against the real thing.
///
/// `#[ignore]` because it needs a running Redis, same convention as any
/// integration test with a live dependency: `cargo test -- --ignored`
/// (or unignored explicitly) with REDIS_URL set, defaulting to
/// redis://localhost:6379 to match local dev / docker compose.
#[cfg(test)]
mod tests {
    use super::*;

    async fn test_connection() -> ConnectionManager {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let client = redis::Client::open(url).expect("invalid REDIS_URL");
        ConnectionManager::new(client)
            .await
            .expect("could not connect to redis -- is it running? see the #[ignore] note above this module")
    }

    async fn key_exists(conn: &mut ConnectionManager, key: &str) -> bool {
        conn.exists(key).await.expect("redis EXISTS failed")
    }

    #[tokio::test]
    #[ignore]
    async fn delete_image_actually_removes_the_key() {
        let mut conn = test_connection().await;
        let job_id = JobId::new();

        store_image(&mut conn, job_id, b"fake image bytes").await.unwrap();
        assert!(key_exists(&mut conn, &image_key(job_id)).await, "key should exist right after store_image");

        // This is the exact call documents.rs makes when a Kafka publish
        // fails after the Redis write already succeeded.
        delete_image(&mut conn, job_id).await;
        assert!(
            !key_exists(&mut conn, &image_key(job_id)).await,
            "the compensating delete must actually remove the orphaned key, not just appear to"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn store_image_refuses_to_overwrite_an_existing_key() {
        let mut conn = test_connection().await;
        let job_id = JobId::new();

        store_image(&mut conn, job_id, b"first write").await.unwrap();
        let second = store_image(&mut conn, job_id, b"second write").await;
        assert!(second.is_err(), "a job_id collision must be a hard error, never a silent overwrite");

        // cleanup
        delete_image(&mut conn, job_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn status_round_trips_and_deleting_the_image_never_touches_it() {
        let mut conn = test_connection().await;
        let job_id = JobId::new();

        let record = JobStatusRecord {
            status: labhsathi_core::status::JobProgress::Queued,
            fields: None,
            error: None,
        };
        set_status(&mut conn, job_id, &record).await.unwrap();
        store_image(&mut conn, job_id, b"fake image bytes").await.unwrap();

        delete_image(&mut conn, job_id).await;

        // The two keys are independent -- deleting the image key must not
        // be able to accidentally take the status key with it. This is
        // what makes "Redis is a cache, not a datastore" concrete rather
        // than a documentation claim (see docs/adr/0001).
        let status = get_status(&mut conn, job_id).await.unwrap();
        assert!(status.is_some(), "status key must survive the image key being deleted");
        assert_eq!(status.unwrap().status, labhsathi_core::status::JobProgress::Queued);
    }
}
