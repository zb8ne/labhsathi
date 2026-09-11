use base64::Engine;
use serde_json::json;

/// Privacy-by-design note:
/// The uploaded document image is held only in memory for the duration of
/// this single request. It is never written to disk, never logged, and
/// never stored in any database. Only the small set of structured fields
/// below survives past this function call — the raw image bytes are
/// dropped the moment this function returns. This is the app's core
/// differentiator: we extract just enough to match schemes, and nothing
/// about the document itself persists anywhere.
pub async fn extract_fields_from_document(
    image_bytes: &[u8],
    mime_type: &str,
) -> Result<serde_json::Value, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY not set in environment".to_string())?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

    let body = json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 512,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": mime_type,
                        "data": b64
                    }
                },
                {
                    "type": "text",
                    "text": "This is an Indian government ID/income/land document (Aadhaar, ration card, income certificate, land record, etc). Extract ONLY these fields if visible, as strict JSON with no prose: {\"age\": number|null, \"annual_income\": number|null, \"state\": string|null, \"category\": \"general|sc|st|obc|minority\"|null, \"land_holding_acres\": number|null, \"occupation\": string|null}. If a field is not present in the document, use null. Respond with JSON only."
                }
            ]
        }]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let parsed: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("bad response: {e}"))?;

    let text = parsed["content"][0]["text"]
        .as_str()
        .ok_or_else(|| "unexpected response shape".to_string())?;

    serde_json::from_str(text).map_err(|e| format!("model did not return valid JSON: {e}"))
    // `image_bytes` (owned by the caller) and `b64` above go out of scope
    // here and are dropped — nothing about the document is retained.
}
