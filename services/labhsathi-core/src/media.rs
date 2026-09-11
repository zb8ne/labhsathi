//! Shared image-type sniffing, used by api-gateway (to validate an upload
//! before it ever touches Redis/Kafka) and ocr-worker (to set the vision
//! API's `media_type` after fetching the bytes back out of Redis).

use crate::events::ImageMimeType;

/// Hard ceiling on a raw upload. Base64 inflates by ~4/3 and the vision API
/// caps a request at 32 MB, so 20 MB of image is the practical limit.
pub const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;

impl ImageMimeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageMimeType::Jpeg => "image/jpeg",
            ImageMimeType::Png => "image/png",
            ImageMimeType::Gif => "image/gif",
            ImageMimeType::Webp => "image/webp",
        }
    }
}

/// Browsers are inconsistent about what they report for a camera-roll photo:
/// `image/jpg`, `application/octet-stream`, or `image/heic` from an iPhone
/// are all common. Sniff the magic bytes instead of trusting the header —
/// the sniff is authoritative; `claimed` only sharpens the error message.
pub fn detect_media_type(bytes: &[u8], claimed: &str) -> Result<ImageMimeType, String> {
    let sniffed = match bytes {
        [0xFF, 0xD8, 0xFF, ..] => Some(ImageMimeType::Jpeg),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some(ImageMimeType::Png),
        [b'G', b'I', b'F', b'8', ..] => Some(ImageMimeType::Gif),
        _ if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" => {
            Some(ImageMimeType::Webp)
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
    let claimed = claimed.split(';').next().unwrap_or("").trim();
    Err(format!(
        "Unsupported image type '{claimed}'. Use JPEG, PNG, GIF or WebP."
    ))
}

/// Validates size + sniffs type in one call — the check every upload path
/// (api-gateway's HTTP handler) should run before writing anything to Redis.
pub fn validate_upload(bytes: &[u8], claimed_content_type: &str) -> Result<ImageMimeType, String> {
    if bytes.is_empty() {
        return Err("Uploaded file was empty.".to_string());
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "Image is {:.1} MB; the limit is {} MB. Please use a smaller photo.",
            bytes.len() as f64 / (1024.0 * 1024.0),
            MAX_IMAGE_BYTES / (1024 * 1024)
        ));
    }
    detect_media_type(bytes, claimed_content_type)
}
