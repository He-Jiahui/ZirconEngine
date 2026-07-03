use std::{fs, path::PathBuf};

use image::{ImageBuffer, ImageFormat, Rgba};

use super::*;

pub(super) fn render_test_output_dir() -> PathBuf {
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should live below repository root")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render");
    fs::create_dir_all(&output_dir).expect("render product test output dir should be writable");
    output_dir
}

pub(super) fn save_scene_velocity_png(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    output_path: &PathBuf,
    label: &str,
) {
    let bytes = framework
        .last_scene_velocity_readback_rg16_float_bytes_for_tests()
        .unwrap_or_else(|| panic!("{label}: scene-velocity RG16Float bytes should be available"));
    let rgba = visualize_scene_velocity_rg16_float_bits(viewport_size, &bytes, label);
    ImageBuffer::<Rgba<u8>, _>::from_raw(viewport_size.x, viewport_size.y, rgba)
        .unwrap_or_else(|| panic!("{label}: velocity PNG dimensions should match RGBA bytes"))
        .save_with_format(output_path, ImageFormat::Png)
        .unwrap_or_else(|error| panic!("{label}: scene-velocity PNG should be writable: {error}"));
}

fn visualize_scene_velocity_rg16_float_bits(
    viewport_size: UVec2,
    bytes: &[u8],
    label: &str,
) -> Vec<u8> {
    assert_eq!(
        bytes.len(),
        (viewport_size.x * viewport_size.y * 4) as usize,
        "{label}: RG16Float readback byte length mismatch",
    );

    let mut decoded = Vec::with_capacity((viewport_size.x * viewport_size.y) as usize);
    let mut max_x = 0u16;
    let mut max_y = 0u16;
    let mut max_payload = 0u16;
    for pixel in bytes.chunks_exact(4) {
        let (x, y) = rg16_float_payload_bits(pixel);
        let payload = x.max(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
        max_payload = max_payload.max(payload);
        decoded.push((x, y, payload));
    }
    assert!(
        max_payload > 0,
        "{label}: expected nonzero scene-velocity bytes"
    );

    let mut rgba = Vec::with_capacity(decoded.len() * 4);
    for (x, y, payload) in decoded {
        rgba.push(scale_channel(x, max_x));
        rgba.push(scale_channel(y, max_y));
        rgba.push(scale_channel(payload, max_payload));
        rgba.push(u8::MAX);
    }
    rgba
}

fn rg16_float_payload_bits(pixel: &[u8]) -> (u16, u16) {
    (
        u16::from_le_bytes([pixel[0], pixel[1]]) & 0x7fff,
        u16::from_le_bytes([pixel[2], pixel[3]]) & 0x7fff,
    )
}

fn scale_channel(value: u16, max_value: u16) -> u8 {
    if max_value == 0 {
        return 0;
    }
    ((value as f32 / max_value as f32) * 255.0).round() as u8
}
