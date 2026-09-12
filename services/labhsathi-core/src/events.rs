//! Kafka event schemas shared between api-gateway (producer of
//! `document.jobs.submitted`, consumer of the job-status read model) and
//! ocr-worker (consumer of `document.jobs.submitted`, producer of
//! `document.jobs.completed`).
//!
//! The privacy boundary is structural, not conventional: `DocumentJobSubmitted`
//! and `DocumentJobCompleted` have no field capable of holding image bytes:
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

/// Closed set, deliberately not a bare `String`. Mirrors the four types
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
/// length doesn't make image bytes impossible to smuggle in; it makes it
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

/// The six fields ocr-worker is allowed to extract: identical in shape to
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

/// Published by ocr-worker to `document.jobs.completed`. Always published,
/// success or failure, so a job is never silently dropped. `fields` is
/// `None` on `JobStatus::Failed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentJobCompleted {
    pub job_id: JobId,
    pub status: JobStatus,
    pub fields: Option<ExtractedFields>,
    pub ts: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_string_accepts_up_to_max_len() {
        let s = "a".repeat(BoundedString::MAX_LEN);
        assert!(BoundedString::new(s).is_some());
    }

    #[test]
    fn bounded_string_rejects_one_byte_over_max_len() {
        let s = "a".repeat(BoundedString::MAX_LEN + 1);
        assert!(BoundedString::new(s).is_none());
    }

    #[test]
    fn bounded_string_deserialize_rejects_oversized_input() {
        // GitHub issue #4: this is the boundary that matters for the
        // privacy claim -- an external input (the vision API's response)
        // that's too long to be a real field value must be rejected at
        // the deserialization boundary, not truncated or silently accepted.
        let too_long = "a".repeat(BoundedString::MAX_LEN + 1);
        let json = serde_json::to_string(&too_long).unwrap();
        let result: Result<BoundedString, _> = serde_json::from_str(&json);
        assert!(result.is_err());
    }

    #[test]
    fn bounded_string_round_trips_through_json() {
        let original = BoundedString::new("Goa").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let back: BoundedString = serde_json::from_str(&json).unwrap();
        assert_eq!(original, back);
    }

    #[test]
    fn extracted_fields_serializes_with_no_binary_capable_field() {
        // Not a type-system proof (see the module doc's honest-limit note),
        // but a regression guard: if someone ever adds a Vec<u8> or
        // serde_json::Value field to ExtractedFields, this test's json key
        // set no longer matches this fixed list and fails loudly.
        let fields = ExtractedFields {
            age: Some(35),
            annual_income: Some(80_000),
            state: BoundedString::new("Goa"),
            category: BoundedString::new("obc"),
            land_holding_acres: Some(2.0),
            occupation: BoundedString::new("Farmer"),
        };
        let value: serde_json::Value = serde_json::to_value(&fields).unwrap();
        let mut keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["age", "annual_income", "category", "land_holding_acres", "occupation", "state"]
        );
    }

    #[test]
    fn document_job_submitted_round_trips_through_json() {
        let original = DocumentJobSubmitted {
            job_id: JobId::new(),
            mime_type: ImageMimeType::Jpeg,
            ts: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: DocumentJobSubmitted = serde_json::from_str(&json).unwrap();
        assert_eq!(original.job_id, back.job_id);
        assert_eq!(original.mime_type, back.mime_type);
        assert_eq!(original.ts, back.ts);
    }

    #[test]
    fn document_job_completed_failed_has_no_fields() {
        let event = DocumentJobCompleted {
            job_id: JobId::new(),
            status: JobStatus::Failed,
            fields: None,
            ts: 0,
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: DocumentJobCompleted = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, JobStatus::Failed);
        assert!(back.fields.is_none());
    }

    #[test]
    fn job_id_round_trips_through_display_and_from_str() {
        let id = JobId::new();
        let parsed: JobId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn job_id_from_str_rejects_garbage() {
        assert!("not-a-uuid".parse::<JobId>().is_err());
    }
}
