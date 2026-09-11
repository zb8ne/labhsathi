use image::imageops::FilterType;
use image::ImageFormat;
use std::io::Cursor;

/// Downsamples to ~1000px on the long edge before the image ever reaches
/// the vision API. Images dominate the token cost of a vision request, and
/// a phone photo at full resolution is mostly waste -- 1000px is plenty to
/// read a printed ID/income document. Always re-encodes to JPEG regardless
/// of source format, since that's the one format guaranteed small.
pub fn downsample_to_jpeg(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("could not decode image: {e}"))?;

    let (w, h) = (img.width(), img.height());
    let long_edge = w.max(h);
    let resized = if long_edge > 1000 {
        // `resize` fits within a 1000x1000 box while preserving aspect
        // ratio -- the box is a ceiling, not a target, so a 1000x1000
        // source stays 1000x1000 and a 400x2000 source becomes 200x1000.
        img.resize(1000, 1000, FilterType::Triangle)
    } else {
        img
    };

    let mut out = Cursor::new(Vec::new());
    resized
        .write_to(&mut out, ImageFormat::Jpeg)
        .map_err(|e| format!("could not re-encode image: {e}"))?;
    Ok(out.into_inner())
}
