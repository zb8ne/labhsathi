//! The Redis-backed job-status read model. api-gateway's `GET
//! /api/documents/{job_id}` reads this; ocr-worker writes it directly as it
//! processes a job. Shared here so both services serialize/deserialize it
//! identically -- this is NOT a Kafka event schema (see events.rs), it's an
//! ephemeral cache record with its own TTL, separate from the raw-image key.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobProgress {
    Queued,
    Processing,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusRecord {
    pub status: JobProgress,
    /// Populated only once `status` is `Done`.
    pub fields: Option<crate::events::ExtractedFields>,
    pub error: Option<String>,
}
