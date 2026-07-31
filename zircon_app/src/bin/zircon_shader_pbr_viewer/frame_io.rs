use std::path::Path;

use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::ViewportFrame;

pub(crate) fn startup_frame(size: UVec2) -> ViewportFrame {
    status_frame(size, [10, 15, 21], [35, 59, 80])
}

pub(crate) fn error_frame(size: UVec2) -> ViewportFrame {
    status_frame(size, [42, 12, 18], [94, 30, 38])
}

pub(crate) fn write_ready_frame_png(
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
        .map_err(|error| format!("screenshot dimensions do not fit usize: {error}"))?
        .ok_or_else(|| "screenshot dimensions overflow".to_owned())?;
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
            format!("create screenshot directory {}: {error}", parent.display())
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
    .map_err(|error| format!("encode screenshot {}: {error}", path.display()))
}

fn status_frame(size: UVec2, top: [u8; 3], bottom: [u8; 3]) -> ViewportFrame {
    let width = size.x.max(1);
    let height = size.y.max(1);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let t = y as f32 / height.saturating_sub(1).max(1) as f32;
        for x in 0..width {
            let shimmer = if ((x / 18) + (y / 18)) & 1 == 0 { 6 } else { 0 };
            rgba.push(lerp_u8(top[0], bottom[0], t).saturating_add(shimmer));
            rgba.push(lerp_u8(top[1], bottom[1], t).saturating_add(shimmer));
            rgba.push(lerp_u8(top[2], bottom[2], t).saturating_add(shimmer));
            rgba.push(255);
        }
    }
    ViewportFrame {
        width,
        height,
        rgba,
        generation: 0,
        capture_report: Default::default(),
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

#[cfg(test)]
mod tests {
    use super::{error_frame, startup_frame, write_ready_frame_png};

    #[test]
    fn status_frames_clamp_zero_dimensions_and_remain_opaque() {
        let frame = startup_frame(zircon_runtime::core::math::UVec2::new(0, 0));

        assert_eq!((frame.width, frame.height), (1, 1));
        assert_eq!(frame.rgba.len(), 4);
        assert_eq!(frame.rgba[3], 255);
        assert_ne!(
            frame.rgba,
            error_frame(zircon_runtime::core::math::UVec2::new(1, 1)).rgba
        );
    }

    #[test]
    fn ready_frame_png_encoder_roundtrips_rgba_pixels() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_ready_frame_{}_{}.png",
            std::process::id(),
            unique
        ));
        let rgba = [
            255, 0, 0, 255, // red
            0, 255, 0, 128, // green with alpha
        ];

        write_ready_frame_png(&path, 2, 1, &rgba).expect("PNG encoding should succeed");
        let decoded = image::open(&path)
            .expect("written Ready-frame PNG should decode")
            .to_rgba8();
        let _ = std::fs::remove_file(&path);

        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.as_raw(), &rgba);
    }

    #[test]
    fn ready_frame_png_encoder_rejects_mismatched_rgba_without_creating_evidence() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_invalid_ready_frame_{}_{}.png",
            std::process::id(),
            unique
        ));

        let error = write_ready_frame_png(&path, 2, 1, &[255, 0, 0, 255])
            .expect_err("a truncated RGBA frame must not produce evidence");

        assert!(error.contains("does not match 2x1 output"));
        assert!(!path.exists());
    }
}
