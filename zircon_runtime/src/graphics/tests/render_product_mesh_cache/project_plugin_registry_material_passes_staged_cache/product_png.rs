use std::{fs, path::PathBuf};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::core::framework::render::CapturedFrame;
use crate::dynamic_api::prewarm_shader_variants;

use super::assertions::{
    assert_registry_material_pass_first_frame_shader_cache_hit,
    assert_registry_material_pass_prewarm_dimensions_written,
    assert_registry_material_pass_prewarm_written,
    assert_registry_material_pass_velocity_frame_shader_cache_hit,
    assert_runtime_shader_cache_root_empty,
};
use super::case::registry_shader_cases;
use super::fixture::submit_registry_material_passes_with_staged_cache_capture;
use super::manifest::{
    registry_material_pass_product_prewarm_manifest, registry_material_pass_runtime_surface_source,
};
use super::shader_cache_test_roots;

const PRODUCT_READBACK_PNG_STATUS: &str = "render_plan08_project_plugin_registry_material_passes_product_readback_png_passed_renderdoc_deferred";

#[test]
#[ignore = "manual product PNG export for Plan 08 project/plugin registry material-pass staged prewarm"]
fn export_project_plugin_registry_material_passes_product_png() {
    assert!(!PRODUCT_READBACK_PNG_STATUS.is_empty());

    let cache_roots = shader_cache_test_roots(
        "zircon_product_project_plugin_registry_material_passes_product_png",
    );
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let case = registry_shader_cases()[0];
    let manifest = registry_material_pass_product_prewarm_manifest(&[case]);
    let registry_shader_source = registry_material_pass_runtime_surface_source();
    let prewarm_report = prewarm_shader_variants(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(prewarm_report.written_count, manifest.variants.len());
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert_registry_material_pass_prewarm_dimensions_written(&prewarm_report);
    assert_registry_material_pass_prewarm_written(&manifest, &prewarm_report, case);

    let launch = submit_registry_material_passes_with_staged_cache_capture(
        case,
        registry_shader_source.as_str(),
        10_201,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
    );
    assert_registry_material_pass_first_frame_shader_cache_hit(
        &launch.first_frame,
        case,
        &prewarm_report,
    );
    assert_registry_material_pass_velocity_frame_shader_cache_hit(
        &launch.velocity_frame,
        case,
        &prewarm_report,
    );
    assert_runtime_shader_cache_root_empty(
        &cache_roots.runtime_root,
        "registry material-pass PNG export should stay read-only against staged cache",
    );

    let first = launch
        .first_capture
        .as_ref()
        .expect("registry material-pass first frame capture");
    let velocity = launch
        .velocity_capture
        .as_ref()
        .expect("registry material-pass velocity frame capture");
    assert_visible_frame(first, "registry material-pass first frame");
    assert_visible_frame(velocity, "registry material-pass velocity frame");

    let output_path = render_test_output_dir()
        .join("runtime_render_plan08_project_plugin_registry_material_passes_20260703.png");
    save_side_by_side_product_frames(first, velocity, &output_path);

    let _ = fs::remove_dir_all(&cache_roots.root);
}

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

pub(super) fn save_side_by_side_product_frames(
    first: &CapturedFrame,
    velocity: &CapturedFrame,
    output_path: &PathBuf,
) {
    assert_eq!(first.width, velocity.width);
    assert_eq!(first.height, velocity.height);

    let separator_width = 1;
    let width = first.width + velocity.width + separator_width;
    let height = first.height;
    let mut rgba = vec![0; (width * height * 4) as usize];
    copy_frame_into(first, width, 0, &mut rgba);
    write_separator(width, height, first.width, &mut rgba);
    copy_frame_into(velocity, width, first.width + separator_width, &mut rgba);

    ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba)
        .expect("registry material-pass PNG dimensions should match RGBA bytes")
        .save_with_format(output_path, ImageFormat::Png)
        .unwrap_or_else(|error| panic!("registry material-pass PNG should be writable: {error}"));
}

fn copy_frame_into(frame: &CapturedFrame, target_width: u32, x_offset: u32, rgba: &mut [u8]) {
    for y in 0..frame.height {
        let source_start = (y * frame.width * 4) as usize;
        let source_end = source_start + (frame.width * 4) as usize;
        let target_start = ((y * target_width + x_offset) * 4) as usize;
        let target_end = target_start + (frame.width * 4) as usize;
        rgba[target_start..target_end].copy_from_slice(&frame.rgba[source_start..source_end]);
    }
}

fn write_separator(width: u32, height: u32, x: u32, rgba: &mut [u8]) {
    for y in 0..height {
        let offset = ((y * width + x) * 4) as usize;
        rgba[offset..offset + 4].copy_from_slice(&[255, 0, 255, 255]);
    }
}

pub(super) fn assert_visible_frame(frame: &CapturedFrame, label: &str) {
    let visible_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 8 || pixel[1] > 8 || pixel[2] > 8)
        .count();
    assert!(
        visible_pixels >= 32,
        "{label} should contain visible product pixels; visible_pixels={visible_pixels}"
    );
}
