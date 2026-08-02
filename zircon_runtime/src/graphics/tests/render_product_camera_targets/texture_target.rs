use super::assertions::{dominant_blue_pixels, dominant_green_pixels, dominant_red_pixels};
use super::camera::{
    camera_target_product_profile, primary_surface_camera_descriptor, texture_camera_descriptor,
    CameraDescriptorTestExt,
};
use super::fixture::RenderFixture;
use super::mesh::colored_mesh_on_layer;

use crate::core::framework::render::{
    CameraRenderType, RenderCameraClear, RenderCameraTargetGraphImportStatus,
    RenderCameraTargetKind, RenderCameraTargetWritebackStatus, RenderCaptureSource,
    RenderFramework, RenderLayerSet,
};
use crate::core::math::{Transform, Vec3, Vec4};

#[test]
fn texture_target_overlay_camera_draws_layered_mesh_over_base_clear() {
    let fixture = RenderFixture::new("graphics_m4_texture_overlay_stack", [0.0, 0.86, 0.12, 1.0]);
    let texture_id = fixture.insert_srgb_render_target_texture(
        "res://tests/camera-target/overlay-layered-composite.texture",
        fixture.viewport_size,
    );
    let overlay_layer = 2;
    let mut extract =
        fixture.frame_extract(vec![texture_overlay_quad(&fixture, 101, overlay_layer)]);
    let base_camera = texture_camera_descriptor(
        1,
        0,
        texture_id,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(1),
        extract.view.camera.clone(),
    )
    .with_stack([2]);
    let overlay_camera = texture_camera_descriptor(
        2,
        0,
        texture_id,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![base_camera, overlay_camera]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("texture target overlay frame should be capturable");
    let stats = framework.query_stats().unwrap();
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    let pixel_count = (frame.width * frame.height) as usize;
    let red_pixels = dominant_red_pixels(&frame.rgba);
    let green_pixels = dominant_green_pixels(&frame.rgba);
    assert!(
        red_pixels > pixel_count / 3,
        "base red clear should remain visible around overlay quad; red={red_pixels}, total={pixel_count}"
    );
    assert!(
        green_pixels > pixel_count / 20,
        "overlay-only green quad should draw over the loaded base target; green={green_pixels}, total={pixel_count}"
    );
}

#[test]
fn texture_target_overlay_camera_converts_linear_final_product_after_composite() {
    let fixture = RenderFixture::new(
        "graphics_m4_texture_overlay_linear_product",
        [0.0, 0.86, 0.12, 1.0],
    );
    let texture_id = fixture.insert_linear_render_target_texture(
        "res://tests/camera-target/overlay-layered-linear-product.texture",
        fixture.viewport_size,
    );
    let overlay_layer = 2;
    let mut extract =
        fixture.frame_extract(vec![texture_overlay_quad(&fixture, 102, overlay_layer)]);
    let base_camera = texture_camera_descriptor(
        1,
        0,
        texture_id,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(1),
        extract.view.camera.clone(),
    )
    .with_stack([2]);
    let overlay_camera = texture_camera_descriptor(
        2,
        0,
        texture_id,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![base_camera, overlay_camera]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("linear texture target overlay frame should be capturable");
    let stats = framework.query_stats().unwrap();
    let (texture_size, texture_rgba) = framework
        .read_output_target_texture_rgba_for_tests(texture_id)
        .unwrap()
        .expect("linear texture target product should remain readable after writeback");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(texture_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureWritebackConversion
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::Converted
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        stats
            .last_camera_target_graph_import
            .conversion_writeback_count,
        1
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::Converted
    );
    assert_eq!(stats.last_camera_target_writeback.converted_count, 1);
    assert_eq!(stats.last_camera_target_writeback.copied_count, 0);

    let pixel_count = (frame.width * frame.height) as usize;
    let frame_red_pixels = dominant_red_pixels(&frame.rgba);
    let frame_green_pixels = dominant_green_pixels(&frame.rgba);
    let texture_red_pixels = dominant_red_pixels(&texture_rgba);
    let texture_green_pixels = dominant_green_pixels(&texture_rgba);
    assert!(
        frame_red_pixels > pixel_count / 3,
        "base red clear should remain visible in converted final product; red={frame_red_pixels}, total={pixel_count}"
    );
    assert!(
        frame_green_pixels > pixel_count / 20,
        "overlay green quad should draw into converted final product; green={frame_green_pixels}, total={pixel_count}"
    );
    assert!(
        texture_red_pixels > pixel_count / 3,
        "converted texture target should keep the base composite; red={texture_red_pixels}, total={pixel_count}"
    );
    assert!(
        texture_green_pixels > pixel_count / 20,
        "converted texture target should include the terminal overlay draw; green={texture_green_pixels}, total={pixel_count}"
    );
}

#[test]
fn texture_target_stack_preserves_composite_when_primary_surface_renders_later() {
    let fixture = RenderFixture::new(
        "graphics_m4_texture_stack_then_primary_surface",
        [0.0, 0.86, 0.12, 1.0],
    );
    let texture_id = fixture.insert_srgb_render_target_texture(
        "res://tests/camera-target/stack-before-primary.texture",
        fixture.viewport_size,
    );
    let overlay_layer = 2;
    let mut extract =
        fixture.frame_extract(vec![texture_overlay_quad(&fixture, 103, overlay_layer)]);
    let texture_base = texture_camera_descriptor(
        1,
        0,
        texture_id,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(1),
        extract.view.camera.clone(),
    )
    .with_stack([2]);
    let texture_overlay = texture_camera_descriptor(
        2,
        0,
        texture_id,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        3,
        1,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        RenderLayerSet::layer(0),
        extract.view.camera.clone(),
    );
    extract.view = extract
        .view
        .with_cameras(vec![texture_base, texture_overlay, primary]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface terminal frame should be capturable");
    let (texture_size, texture_rgba) = framework
        .read_output_target_texture_rgba_for_tests(texture_id)
        .unwrap()
        .expect("texture target stack should remain prepared after primary surface submit");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(texture_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let pixel_count = (frame.width * frame.height) as usize;
    let frame_blue_pixels = dominant_blue_pixels(&frame.rgba);
    let frame_red_pixels = dominant_red_pixels(&frame.rgba);
    let frame_green_pixels = dominant_green_pixels(&frame.rgba);
    assert!(
        frame_blue_pixels > pixel_count * 3 / 4
            && frame_red_pixels < pixel_count / 10
            && frame_green_pixels < pixel_count / 10,
        "later PrimarySurface Base should own the viewport capture without overwriting the earlier texture target; blue={frame_blue_pixels}, red={frame_red_pixels}, green={frame_green_pixels}, total={pixel_count}"
    );

    let texture_red_pixels = dominant_red_pixels(&texture_rgba);
    let texture_green_pixels = dominant_green_pixels(&texture_rgba);
    let texture_blue_pixels = dominant_blue_pixels(&texture_rgba);
    assert!(
        texture_red_pixels > pixel_count / 3,
        "texture target stack should keep the red Base clear after a later PrimarySurface camera; red={texture_red_pixels}, blue={texture_blue_pixels}, total={pixel_count}"
    );
    assert!(
        texture_green_pixels > pixel_count / 20,
        "texture target stack should keep the green Overlay mesh after a later PrimarySurface camera; green={texture_green_pixels}, total={pixel_count}"
    );
    assert!(
        texture_blue_pixels < pixel_count / 10,
        "later PrimarySurface blue clear must not leak into the custom texture target; blue={texture_blue_pixels}, total={pixel_count}"
    );
}

fn texture_overlay_quad(
    fixture: &RenderFixture,
    node_id: u64,
    overlay_layer: u32,
) -> crate::core::framework::render::RenderMeshSnapshot {
    colored_mesh_on_layer(
        node_id,
        fixture.model,
        fixture.material,
        Transform {
            scale: Vec3::new(0.75, 0.75, 1.0),
            ..Transform::default()
        },
        Vec4::ONE,
        overlay_layer,
    )
}
