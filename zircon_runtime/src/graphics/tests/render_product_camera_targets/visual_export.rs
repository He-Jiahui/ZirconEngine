use std::path::PathBuf;

use crate::core::framework::render::{
    CameraRenderType, RenderCameraClear, RenderFramework, RenderLayerSet, RenderViewportRect,
};
use crate::core::math::{UVec2, Vec4};

use super::assertions::{
    dominant_blue_pixels_in_region, dominant_blue_pixels_in_rgba_region,
    dominant_green_pixels_in_region, dominant_green_pixels_in_rgba_region,
    dominant_red_pixels_in_region, dominant_red_pixels_in_rgba_region, RenderViewportRegion,
};
use super::camera::{
    camera_target_product_profile, primary_surface_camera_descriptor, texture_camera_descriptor,
    CameraDescriptorTestExt,
};
use super::fixture::RenderFixture;
use super::mesh::{overlay_mesh, sampled_fullscreen_mesh};

const CAMERA_TARGET_WGPU_PNG: &str = "plan09_camera_custom_target_overlay_wgpu_20260718.png";

#[test]
#[ignore = "writes Plan 09 WGPU framebuffer evidence under docs/tests/runtime/render"]
fn export_camera_custom_target_overlay_wgpu_png() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_visual_evidence",
        [1.0, 1.0, 1.0, 1.0],
    );
    let target_uri = "res://tests/camera-target/plan09-visual-evidence.texture";
    let target = fixture.insert_srgb_render_target_texture(target_uri, fixture.viewport_size);
    let sampled_material = fixture.insert_texture_sampling_material(
        "res://materials/plan09-visual-evidence-sample.zmaterial",
        target_uri,
    );

    let mut extract = fixture.frame_extract(vec![
        overlay_mesh(
            9_701,
            fixture.model,
            fixture.material,
            19,
            Vec4::new(0.0, 1.0, 0.0, 1.0),
        ),
        sampled_fullscreen_mesh(9_702, fixture.model, sampled_material),
    ]);
    let half_width = fixture.viewport_size.x / 2;
    let left_viewport =
        RenderViewportRect::new(UVec2::ZERO, UVec2::new(half_width, fixture.viewport_size.y));
    let right_viewport = RenderViewportRect::new(
        UVec2::new(half_width, 0),
        UVec2::new(
            fixture.viewport_size.x - half_width,
            fixture.viewport_size.y,
        ),
    );
    let right_base = texture_camera_descriptor(
        9_801,
        -30,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        true,
        RenderLayerSet::layer(20),
        extract.view.camera.clone(),
    )
    .with_viewport(right_viewport);
    let left_base = texture_camera_descriptor(
        9_802,
        -20,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(21),
        extract.view.camera.clone(),
    )
    .with_viewport(left_viewport)
    .with_stack([9_803]);
    let left_overlay = texture_camera_descriptor(
        9_803,
        -19,
        target,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(19),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        9_804,
        10,
        RenderCameraClear::Color(Vec4::new(0.015, 0.015, 0.015, 1.0)),
        RenderLayerSet::layer(0),
        extract.view.camera.clone(),
    );
    extract.view = extract
        .view
        .with_cameras(vec![right_base, left_base, left_overlay, primary]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("Plan 09 PrimarySurface evidence frame should be capturable");
    let stats = framework.query_stats().unwrap();
    let (target_size, target_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target)
        .unwrap()
        .expect("Plan 09 custom target should remain readable");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(stats.last_camera_loop_submission_count, 4);
    assert_eq!(target_size, fixture.viewport_size);
    assert_camera_target_visual_regions(&frame, &target_rgba, target_size);

    let output = repository_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
        .join(CAMERA_TARGET_WGPU_PNG);
    std::fs::create_dir_all(output.parent().expect("render evidence directory")).unwrap();
    image::save_buffer_with_format(
        &output,
        &frame.rgba,
        frame.width,
        frame.height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();
    assert!(
        output.is_file(),
        "missing Plan 09 visual evidence: {}",
        output.display()
    );
}

fn assert_camera_target_visual_regions(
    frame: &crate::core::framework::render::CapturedFrame,
    target_rgba: &[u8],
    target_size: UVec2,
) {
    let left_overlay = RenderViewportRegion::new(UVec2::new(24, 28), UVec2::new(40, 64));
    let left_edge = RenderViewportRegion::new(UVec2::new(4, 20), UVec2::new(18, 80));
    let right = RenderViewportRegion::new(UVec2::new(84, 28), UVec2::new(48, 64));
    let left_overlay_pixels = (left_overlay.size.x * left_overlay.size.y) as usize;
    let left_edge_pixels = (left_edge.size.x * left_edge.size.y) as usize;
    let right_pixels = (right.size.x * right.size.y) as usize;

    assert!(
        dominant_green_pixels_in_rgba_region(target_rgba, target_size, left_overlay)
            > left_overlay_pixels / 10,
        "custom target must retain the green Overlay inside the left Base viewport"
    );
    assert!(
        dominant_red_pixels_in_rgba_region(target_rgba, target_size, left_edge)
            > left_edge_pixels / 3,
        "custom target must retain the red Base clear outside the Overlay mesh"
    );
    assert!(
        dominant_blue_pixels_in_rgba_region(target_rgba, target_size, right) > right_pixels / 2,
        "custom target must retain the independent blue right Base viewport"
    );
    assert!(
        dominant_green_pixels_in_region(frame, left_overlay) > left_overlay_pixels / 10,
        "PrimarySurface must sample the green left Overlay result"
    );
    assert!(
        dominant_red_pixels_in_region(frame, left_edge) > left_edge_pixels / 3,
        "PrimarySurface must sample the red left Base outside the Overlay mesh"
    );
    assert!(
        dominant_blue_pixels_in_region(frame, right) > right_pixels / 3
            && dominant_green_pixels_in_region(frame, right) < right_pixels / 10,
        "PrimarySurface must sample the blue right Base without Overlay contamination"
    );
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should live below the repository root")
        .to_path_buf()
}
