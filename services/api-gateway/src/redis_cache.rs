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
