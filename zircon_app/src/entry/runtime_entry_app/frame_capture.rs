use std::path::Path;

pub(super) fn write_runtime_frame_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let expected_len = width
        .checked_mul(height)
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .map(usize::try_from)
        .transpose()
        .map_err(|error| format!("frame dimensions do not fit usize: {error}"))?
        .ok_or_else(|| "frame dimensions overflow".to_owned())?;
    if rgba.len() != expected_len {
        return Err(format!(
            "frame RGBA length {} does not match {width}x{height} output",
            rgba.len()
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create frame capture directory {}: {error}",
                parent.display()
            )
        })?;
    }
    image::save_buffer_with_format(
        path,
        rgba,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|error| format!("encode frame capture {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::write_runtime_frame_png;

    #[test]
    fn runtime_frame_png_encoder_roundtrips_rgba_pixels() {
        let path = std::env::temp_dir().join(format!(
            "zircon_runtime_frame_capture_{}_{}.png",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos()
        ));
        let rgba = [
            255, 0, 0, 255, // red
            0, 255, 0, 128, // green with alpha
        ];

        write_runtime_frame_png(&path, 2, 1, &rgba).expect("frame capture PNG should encode");
        let decoded = image::open(&path)
            .expect("written frame capture PNG should decode")
            .to_rgba8();
        let _ = std::fs::remove_file(&path);

        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.as_raw(), &rgba);
    }

    #[test]
    fn runtime_frame_png_encoder_rejects_mismatched_rgba_without_writing_evidence() {
        let path = std::env::temp_dir().join(format!(
            "zircon_runtime_frame_capture_invalid_{}_{}.png",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos()
        ));

        let error = write_runtime_frame_png(&path, 2, 1, &[255, 0, 0, 255])
            .expect_err("truncated RGBA frame must not produce PNG evidence");

        assert!(error.contains("does not match 2x1 output"));
        assert!(!path.exists());
    }
}
