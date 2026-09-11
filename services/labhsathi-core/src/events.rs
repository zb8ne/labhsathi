//! Kafka event schemas shared between api-gateway (producer of
//! `document.jobs.submitted`, consumer of the job-status read model) and
//! ocr-worker (consumer of `document.jobs.submitted`, producer of
//! `document.jobs.completed`).
//!
//! The privacy boundary is structural, not conventional: `DocumentJobSubmitted`
//! and `DocumentJobCompleted` have no field capable of holding image bytes —
//! no `Vec<u8>`, no `serde_json::Value`, nothing unbounded. See
//! docs/adr/0001-event-driven-document-pipeline.md for the honest limit on
//! this claim (a `BoundedString` is not literally incapable of misuse, just
//! impractical for image data and structurally distinct from a byte field).

use serde::{Deserialize, Serialize};
use std::fmt;

pub const TOPIC_DOCUMENT_JOBS_SUBMITTED: &str = "document.jobs.submitted";
pub const TOPIC_DOCUMENT_JOBS_COMPLETED: &str = "document.jobs.completed";

/// Newtype over Uuid so a job id can never be confused with, or silently
/// widened into, an arbitrary string field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub uuid::Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for JobId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(uuid::Uuid::parse_str(s)?))
    }
}

/// Closed set — deliberately not a bare `String`. Mirrors the four types
/// the vision API accepts (see labhsathi-core::media).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageMimeType {
    Jpeg,
    Png,
    Gif,
    Webp,
}

/// A short, length-capped string for the handful of extracted-field values
/// that are still free text (state name, occupation, etc). Bounding the
/// length doesn't make image bytes impossible to smuggle in — it makes it
/// pointless: a few dozen bytes can't usefully carry a document image, so
/// the type is a practical reinforcement of the schema-level guarantee, not
/// the guarantee itself. The guarantee itself is the *absence* of any field
/// sized or typed for arbitrary binary data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedString(String);

impl BoundedString {
    pub const MAX_LEN: usize = 64;

    pub fn new(s: impl Into<String>) -> Option<Self> {
        let s = s.into();
        if s.len() <= Self::MAX_LEN {
            Some(Self(s))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for BoundedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BoundedString::new(s).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "string exceeds BoundedString::MAX_LEN ({} bytes)",
                Self::MAX_LEN
            ))
        })
    }
}

/// Published by api-gateway to `document.jobs.submitted` after the raw image
/// is already written to Redis under `job_id` with a 60s TTL. Note what is
/// NOT here: no image bytes, no base64, no arbitrary blob field of any kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentJobSubmitted {
    pub job_id: JobId,
    pub mime_type: ImageMimeType,
    pub ts: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Success,
    Failed,
}

/// The six fields ocr-worker is allowed to extract — identical in shape to
/// the extraction schema sent to the vision API (see ocr-worker::extract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFields {
    pub age: Option<u32>,
    pub annual_income: Option<u64>,
    pub state: Option<BoundedString>,
    pub category: Option<BoundedString>,
    pub land_holding_acres: Option<f64>,
    pub occupation: Option<BoundedString>,
}

/// Published by ocr-worker to `document.jobs.completed`. Always published —
/// success or failure — so a job is never silently dropped. `fields` is
/// `None` on `JobStatus::Failed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentJobCompleted {
    pub job_id: JobId,
    pub status: JobStatus,
    pub fields: Option<ExtractedFields>,
    pub ts: i64,
}
