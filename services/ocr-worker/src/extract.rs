use base64::Engine;
use labhsathi_core::events::ExtractedFields;
use serde_json::json;

/// Only these six fields are ever pulled out of a document. Enforced
/// server-side by the vision API via structured outputs, so the model
/// physically cannot return more -- mirrors `ExtractedFields` in
/// labhsathi-core::events exactly.
fn extraction_schema() -> serde_json::Value {
    let nullable = |t: &str| json!({"anyOf": [{"type": t}, {"type": "null"}]});
    json!({
        "type": "object",
        "properties": {
            "age": nullable("integer"),
            "annual_income": nullable("integer"),
            "state": nullable("string"),
            "category": {"anyOf": [
                {"type": "string", "enum": ["general", "sc", "st", "obc", "minority"]},
                {"type": "null"}
            ]},
            "land_holding_acres": nullable("number"),
            "occupation": nullable("string")
        },
        "required": [
            "age", "annual_income", "state",
            "category", "land_holding_acres", "occupation"
        ],
        "additionalProperties": false
    })
}

/// Calls Claude's vision API once on an already-downsampled JPEG and
/// returns the six structured fields. The image bytes passed in are owned
/// by the caller and go out of scope with it -- nothing here writes them
/// anywhere but this one outbound request body.
///
/// Takes a `&reqwest::Client` rather than constructing one per call --
/// reqwest's client owns a connection pool, and building a fresh one per
/// request throws that away every time. The caller builds one client at
/// startup (see consumer.rs) with a request timeout configured; without
/// one, GitHub issue #7: a hung call never returns, and since extraction
/// runs under a bounded semaphore (MAX_CONCURRENT_EXTRACTIONS permits),
/// enough hung calls exhaust every permit and the worker stops making
/// progress on anything, forever.
///
/// Model/param choices are deliberate, not defaults:
/// - claude-haiku-4-5, not a larger model -- this is pure structured
///   extraction with no reasoning required, and Haiku is far cheaper.
/// - No `effort` key: Haiku 4.5 errors if one is set.
/// - No `thinking` key: Haiku 4.5 has no adaptive-thinking mode, and this
///   task needs none.
/// - No `fallbacks`: unconfirmed whether refusal-fallback routing is
///   supported on Haiku 4.5 (the docs available while building this only
///   confirmed it for Fable 5.1) -- don't ship an unverified beta flag.
///   TODO(verify): check this against the live API before re-adding.
pub async fn extract_fields_from_jpeg(
    client: &reqwest::Client,
    jpeg_bytes: &[u8],
) -> Result<ExtractedFields, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY not set in environment".to_string())?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(jpeg_bytes);

    let body = json!({
        "model": "claude-haiku-4-5",
        "max_tokens": 1024,
        "output_config": {
            "format": {"type": "json_schema", "schema": extraction_schema()}
        },
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/jpeg",
                        "data": b64
                    }
                },
                {
                    "type": "text",
                    "text": "This is an Indian government ID, income, or land document (Aadhaar, ration card, income certificate, land record, etc). Extract only the fields defined by the response schema. Use null for anything not clearly visible in the document -- never guess or infer a value that is not printed. Report annual_income in rupees per year: if the document states a monthly figure, multiply by 12."
                }
            ]
        }]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "vision API request timed out".to_string()
            } else {
                format!("request failed: {e}")
            }
        })?;

    let status = resp.status();
    let parsed: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("bad response: {e}"))?;

    if !status.is_success() {
        let msg = parsed["error"]["message"].as_str().unwrap_or("unknown API error");
        return Err(format!("API error ({status}): {msg}"));
    }

    match parsed["stop_reason"].as_str() {
        Some("refusal") => return Err("model declined to read this document".to_string()),
        Some("max_tokens") => return Err("response was cut short before completing".to_string()),
        _ => {}
    }

    // Thinking blocks (if any ever appear) can precede the text block, so
    // select by type rather than assuming content[0].
    let text = parsed["content"]
        .as_array()
        .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text").and_then(|b| b["text"].as_str()))
        .ok_or_else(|| "no text block in response".to_string())?;

    let raw: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("model did not return valid JSON: {e}"))?;

    fields_from_raw_json(raw)
}

/// The vision API's response uses plain strings/numbers per the JSON
/// schema above; this converts into `ExtractedFields`, where the free-text
/// fields are the length-capped `BoundedString`. A field that somehow
/// arrives longer than `BoundedString::MAX_LEN` (should never happen given
/// the extraction schema, but the API is an external boundary) is dropped
/// to `None` rather than failing the whole extraction.
fn fields_from_raw_json(raw: serde_json::Value) -> Result<ExtractedFields, String> {
    use labhsathi_core::events::BoundedString;

    let bounded = |v: &serde_json::Value| -> Option<BoundedString> {
        v.as_str().and_then(BoundedString::new)
    };

    Ok(ExtractedFields {
        age: raw["age"].as_u64().map(|n| n as u32),
        annual_income: raw["annual_income"].as_u64(),
        state: bounded(&raw["state"]),
        category: bounded(&raw["category"]),
        land_holding_acres: raw["land_holding_acres"].as_f64(),
        occupation: bounded(&raw["occupation"]),
    })
}
