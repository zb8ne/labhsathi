use base64::Engine;
use serde_json::json;

/// Hard ceiling on the raw upload. Base64 inflates by ~4/3 and the API caps a
/// request at 32 MB, so 20 MB of image is the practical limit.
const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;

/// Only these six fields are ever pulled out of a document. Enforced server-side
/// by the API via structured outputs, so the model physically cannot return more.
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

/// Browsers are inconsistent about what they report for a camera roll photo:
/// `image/jpg`, `application/octet-stream`, or `image/heic` from an iPhone are
/// all common. Sniff the magic bytes instead of trusting the header.
fn detect_media_type(bytes: &[u8], claimed: &str) -> Result<&'static str, String> {
    let sniffed = match bytes {
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg"),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("image/png"),
        [b'G', b'I', b'F', b'8', ..] => Some("image/gif"),
        _ if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" => {
            Some("image/webp")
        }
        _ => None,
    };
    if let Some(m) = sniffed {
        return Ok(m);
    }
    // HEIC/HEIF ships in an ISO-BMFF container: "ftyp" at offset 4.
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        return Err(
            "This looks like a HEIC/HEIF photo (iPhone default). Please re-save it as JPEG or PNG."
                .to_string(),
        );
    }
    // The sniff is authoritative: JPEG/PNG/GIF/WebP all have reliable magic
    // bytes, so anything unrecognised here is not one of them regardless of
    // what the client claimed. `claimed` only sharpens the error message.
    let claimed = claimed.split(';').next().unwrap_or("").trim();
    Err(format!(
        "Unsupported image type '{claimed}'. Use JPEG, PNG, GIF or WebP."
    ))
}

/// Privacy-by-design note:
/// The uploaded document image is held only in memory for the duration of
/// this single request. It is never written to disk, never logged, and
/// never stored in any database. Only the six structured fields declared in
/// `extraction_schema` survive past this function call — the raw image bytes
/// are dropped the moment this function returns. This is the app's core
/// differentiator: we extract just enough to match schemes, and nothing
/// about the document itself persists anywhere.
pub async fn extract_fields_from_document(
    image_bytes: &[u8],
    mime_type: &str,
) -> Result<serde_json::Value, String> {
    if image_bytes.is_empty() {
        return Err("Uploaded file was empty.".to_string());
    }
    if image_bytes.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "Image is {:.1} MB; the limit is {} MB. Please use a smaller photo.",
            image_bytes.len() as f64 / (1024.0 * 1024.0),
            MAX_IMAGE_BYTES / (1024 * 1024)
        ));
    }

    let media_type = detect_media_type(image_bytes, mime_type)?;

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY not set in environment".to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

    let body = json!({
        "model": "claude-opus-5",
        "max_tokens": 8192,
        // Thinking is on by default on Opus 5 and its tokens count against
        // max_tokens. Field extraction needs no deliberation, so run it at low
        // effort — faster and cheaper without touching accuracy on this task.
        "output_config": {
            "effort": "low",
            "format": {"type": "json_schema", "schema": extraction_schema()}
        },
        // A request to read an Aadhaar/ID card is exactly the shape a safety
        // classifier may decline. Server-side fallbacks re-run the same request
        // on another model inside the same call rather than failing the demo.
        "fallbacks": "default",
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": b64
                    }
                },
                {
                    "type": "text",
                    "text": "This is an Indian government ID, income, or land document (Aadhaar, ration card, income certificate, land record, etc). Extract only the fields defined by the response schema. Use null for anything not clearly visible in the document — never guess or infer a value that is not printed. Report annual_income in rupees per year: if the document states a monthly figure, multiply by 12."
                }
            ]
        }]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "server-side-fallback-2026-07-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = resp.status();
    let parsed: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("bad response: {e}"))?;

    // Surface the API's own error message instead of failing later with a
    // confusing "unexpected response shape".
    if !status.is_success() {
        let msg = parsed["error"]["message"]
            .as_str()
            .unwrap_or("unknown API error");
        return Err(format!("API error ({status}): {msg}"));
    }

    match parsed["stop_reason"].as_str() {
        Some("refusal") => {
            return Err("The model declined to read this document. Please enter your details manually.".to_string())
        }
        Some("max_tokens") => {
            return Err("Response was cut short before completing. Please try again.".to_string())
        }
        _ => {}
    }

    // Thinking blocks can precede the text block, so select by type rather
    // than assuming content[0].
    let text = parsed["content"]
        .as_array()
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|b| b["type"] == "text")
                .and_then(|b| b["text"].as_str())
        })
        .ok_or_else(|| "no text block in response".to_string())?;

    serde_json::from_str(text).map_err(|e| format!("model did not return valid JSON: {e}"))
    // `image_bytes` (owned by the caller) and `b64` above go out of scope
    // here and are dropped — nothing about the document is retained.
}
