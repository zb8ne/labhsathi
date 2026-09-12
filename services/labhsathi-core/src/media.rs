//! Shared image-type sniffing, used by api-gateway (to validate an upload
//! before it ever touches Redis/Kafka) and ocr-worker (to set the vision
//! API's `media_type` after fetching the bytes back out of Redis).

use crate::events::ImageMimeType;
use std::io::Cursor;

/// Hard ceiling on a raw upload. Base64 inflates by ~4/3 and the vision API
/// caps a request at 32 MB, so 20 MB of image is the practical limit.
pub const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;

/// Ceiling on decoded pixel count -- ~50 megapixels, generous for any real
/// photo (phone cameras top out well under this) but far below what a
/// decompression bomb claims. A few hundred bytes of compressed PNG/GIF
/// can declare dimensions that decode into gigabytes of pixel data; this
/// check reads only the header (no full decode) so it's cheap to run on
/// every upload before anything expensive happens.
pub const MAX_DECODED_PIXELS: u64 = 50_000_000;

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
/// are all common. Sniff the magic bytes instead of trusting the header:
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

/// Reads just enough of the header to know the decoded width/height,
/// without decoding any pixel data -- safe to run even on a hostile input.
fn check_decoded_dimensions(bytes: &[u8]) -> Result<(), String> {
    let (width, height) = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("could not read image header: {e}"))?
        .into_dimensions()
        .map_err(|_| "Could not read this image's dimensions.".to_string())?;

    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_DECODED_PIXELS {
        return Err(format!(
            "This image decodes to {width}x{height} ({:.0} megapixels), which is above the {:.0} megapixel limit.",
            pixels as f64 / 1_000_000.0,
            MAX_DECODED_PIXELS as f64 / 1_000_000.0
        ));
    }
    Ok(())
}

/// Validates size, sniffs type, and rejects decompression bombs -- the
/// check every upload path (api-gateway's HTTP handler) should run before
/// writing anything to Redis. Order matters: size and type are checked
/// first since they're free; the dimension check (a header parse) only
/// runs on something that already looks like a plausible image.
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
    let mime_type = detect_media_type(bytes, claimed_content_type)?;
    check_decoded_dimensions(bytes)?;
    Ok(mime_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, ImageFormat, Rgb};

    /// A tiny real PNG (valid pixel data, small dimensions) for happy-path
    /// tests -- built with the `image` crate rather than hand-authored
    /// bytes, so a change to the encoder can't silently desync the fixture
    /// from what a real upload actually looks like.
    fn tiny_png() -> Vec<u8> {
        let img: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_fn(4, 4, |_, _| Rgb([255, 0, 0]));
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png).unwrap();
        buf
    }

    #[test]
    fn detect_media_type_reads_real_magic_bytes() {
        assert_eq!(detect_media_type(&tiny_png(), "image/png").unwrap(), ImageMimeType::Png);
    }

    #[test]
    fn detect_media_type_is_authoritative_over_claimed_header() {
        // The sniff wins even when the client claims something else --
        // this is real PNG data with a lying Content-Type.
        assert_eq!(
            detect_media_type(&tiny_png(), "application/octet-stream").unwrap(),
            ImageMimeType::Png
        );
    }

    #[test]
    fn detect_media_type_rejects_heic_with_a_specific_message() {
        // Minimal ISO-BMFF "ftyp" box header, enough to trigger the HEIC
        // detection path without needing a full valid HEIC file.
        let heic_like = b"\x00\x00\x00\x18ftypheic\x00\x00\x00\x00";
        let err = detect_media_type(heic_like, "image/heic").unwrap_err();
        assert!(err.contains("HEIC"), "expected a HEIC-specific message, got: {err}");
    }

    #[test]
    fn detect_media_type_rejects_unrecognized_bytes() {
        let err = detect_media_type(b"not an image at all", "text/plain").unwrap_err();
        assert!(err.contains("text/plain"));
    }

    #[test]
    fn validate_upload_rejects_empty() {
        assert!(validate_upload(&[], "image/png").is_err());
    }

    #[test]
    fn validate_upload_rejects_over_byte_limit() {
        let oversized = vec![0xFFu8; MAX_IMAGE_BYTES + 1];
        let err = validate_upload(&oversized, "image/jpeg").unwrap_err();
        assert!(err.contains("MB"));
    }

    #[test]
    fn validate_upload_accepts_a_real_small_png() {
        assert_eq!(validate_upload(&tiny_png(), "image/png").unwrap(), ImageMimeType::Png);
    }

    #[test]
    fn validate_upload_rejects_a_real_decompression_bomb() {
        // GitHub issue #6, reproduced for real rather than asserted in the
        // abstract: an 8000x8000 solid-color PNG decodes to 64 megapixels
        // (over MAX_DECODED_PIXELS) but PNG's DEFLATE compression crushes
        // a single flat color down to a few KB on disk -- exactly the
        // small-file/huge-decode shape a decompression bomb has, built
        // with the real encoder rather than hand-crafted bytes.
        let bomb: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_fn(8000, 8000, |_, _| Rgb([10, 10, 10]));
        let mut buf = Vec::new();
        bomb.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png).unwrap();

        // Raw pixel data for this image would be 8000*8000*3 = 192MB; the
        // compressed file is a small fraction of that (still under
        // MAX_IMAGE_BYTES, so it's the dimension check, not the byte-size
        // check, that has to catch it) -- that gap is the actual bomb shape.
        assert!(buf.len() < MAX_IMAGE_BYTES, "must pass the byte-size check to prove the dimension check is what catches it");

        let err = validate_upload(&buf, "image/png").unwrap_err();
        assert!(err.contains("megapixel"), "expected a megapixel-limit rejection, got: {err}");
    }
}
