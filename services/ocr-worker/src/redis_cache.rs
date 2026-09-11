use labhsathi_core::events::JobId;
use labhsathi_core::status::JobStatusRecord;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

fn image_key(job_id: JobId) -> String {
    format!("labhsathi:doc:{job_id}")
}

fn status_key(job_id: JobId) -> String {
    format!("labhsathi:job:{job_id}")
}

const STATUS_TTL_SECS: u64 = 300;

/// Atomic fetch-and-delete: the image key is gone the instant this returns,
/// whether or not extraction goes on to succeed. `GETDEL` (not GET then
/// DEL) closes the race where two workers could otherwise both read the
/// same image before either deletes it.
///
/// `None` means either the job was already consumed (a redelivery after a
/// crash) or the 60s TTL beat the worker to it -- both are legitimate,
/// non-corrupt states the caller must handle explicitly, not an error.
pub async fn fetch_and_delete_image(
    conn: &mut ConnectionManager,
    job_id: JobId,
) -> Result<Option<Vec<u8>>, String> {
    conn.get_del(image_key(job_id))
        .await
        .map_err(|e| format!("redis GETDEL failed: {e}"))
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
