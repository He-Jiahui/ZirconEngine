use super::assertions::{
    average_channel_in_region, dominant_green_pixels, dominant_green_pixels_in_region,
    dominant_red_pixels, dominant_red_pixels_in_region, RenderViewportRegion,
};
use super::camera::{
    camera_target_product_profile, primary_surface_camera_descriptor,
    primary_surface_stack_camera_descriptor, CameraDescriptorTestExt,
};
use super::fixture::RenderFixture;
use super::mesh::colored_mesh_on_layer;

use crate::core::framework::render::{
    CameraRenderType, CapturedFrame, RenderCameraClear, RenderCameraTargetKind,
    RenderCaptureSource, RenderFramework, RenderLayerSet, RenderViewportRect,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};

#[test]
fn render_product_camera_render_order_swap_changes_composite() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_primary_base_order",
        [1.0, 1.0, 1.0, 1.0],
    );
    let green_last = capture_primary_surface_render_order_scene(&fixture, 0, 1);
    let red_last = capture_primary_surface_render_order_scene(&fixture, 1, 0);
    let pixel_count = (fixture.viewport_size.x * fixture.viewport_size.y) as usize;

    let green_last_green = dominant_green_pixels(&green_last.rgba);
    let green_last_red = dominant_red_pixels(&green_last.rgba);
    assert!(
        green_last_green > pixel_count * 3 / 4 && green_last_red < pixel_count / 10,
        "later green Base camera should own the final primary surface composite; green={green_last_green}, red={green_last_red}, total={pixel_count}"
    );

    let red_last_red = dominant_red_pixels(&red_last.rgba);
    let red_last_green = dominant_green_pixels(&red_last.rgba);
    assert!(
        red_last_red > pixel_count * 3 / 4 && red_last_green < pixel_count / 10,
        "later red Base camera should own the final primary surface composite after render_order swap; red={red_last_red}, green={red_last_green}, total={pixel_count}"
    );
}

fn capture_primary_surface_render_order_scene(
    fixture: &RenderFixture,
    red_order: i32,
    green_order: i32,
) -> CapturedFrame {
    let mut extract = fixture.frame_extract(Vec::new());
    let red_camera = primary_surface_camera_descriptor(
        904,
        red_order,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        RenderLayerSet::default(),
        extract.view.camera.clone(),
    );
    let green_camera = primary_surface_camera_descriptor(
        905,
        green_order,
        RenderCameraClear::Color(Vec4::new(0.0, 1.0, 0.0, 1.0)),
        RenderLayerSet::default(),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![red_camera, green_camera]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface terminal frame should be capturable");
    framework.destroy_viewport(viewport).unwrap();
    frame
}

#[test]
fn render_product_overlay_stack_composites_over_base() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_primary_overlay_clear_depth",
        [1.0, 1.0, 1.0, 1.0],
    );
    let depth_loaded = capture_primary_surface_overlay_depth_scene(&fixture, false);
    let depth_cleared = capture_primary_surface_overlay_depth_scene(&fixture, true);
    let sample = RenderViewportRegion::new(
        UVec2::new(
            fixture.viewport_size.x / 2 - 12,
            fixture.viewport_size.y / 2 - 12,
        ),
        UVec2::new(24, 24),
    );

    let loaded_red = average_channel_in_region(&depth_loaded, sample, 0);
    let loaded_green = average_channel_in_region(&depth_loaded, sample, 1);
    assert!(
        loaded_red > 128.0 && loaded_green < 64.0,
        "Overlay clear_depth=false should preserve Base depth and keep the farther green overlay behind the red Base mesh; red={loaded_red:.2}, green={loaded_green:.2}"
    );

    let cleared_red = average_channel_in_region(&depth_cleared, sample, 0);
    let cleared_green = average_channel_in_region(&depth_cleared, sample, 1);
    assert!(
        cleared_green > 128.0 && cleared_red < 64.0,
        "Overlay clear_depth=true should clear Base depth and let the farther green overlay replace the center pixels; red={cleared_red:.2}, green={cleared_green:.2}"
    );
}

fn capture_primary_surface_overlay_depth_scene(
    fixture: &RenderFixture,
    overlay_clear_depth: bool,
) -> CapturedFrame {
    let base_layer = 1;
    let overlay_layer = 2;
    let mut extract = fixture.frame_extract(vec![
        colored_mesh_on_layer(
            906,
            fixture.model,
            fixture.material,
            Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                scale: Vec3::new(1.1, 1.1, 1.0),
                ..Transform::default()
            },
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            base_layer,
        ),
        colored_mesh_on_layer(
            907,
            fixture.model,
            fixture.material,
            Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(0.82, 0.82, 1.0),
                ..Transform::default()
            },
            Vec4::new(0.0, 1.0, 0.0, 1.0),
            overlay_layer,
        ),
    ]);
    let base_camera = primary_surface_stack_camera_descriptor(
        906,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        true,
        RenderLayerSet::layer(base_layer),
        extract.view.camera.clone(),
    )
    .with_stack([907]);
    let overlay_camera = primary_surface_stack_camera_descriptor(
        907,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        overlay_clear_depth,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![base_camera, overlay_camera]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface terminal frame should be capturable");
    framework.destroy_viewport(viewport).unwrap();
    frame
}

#[test]
fn render_product_split_screen_viewports() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_split_screen_viewports",
        [1.0, 1.0, 1.0, 1.0],
    );
    let half_width = fixture.viewport_size.x / 2;
    let left_half =
        RenderViewportRect::new(UVec2::ZERO, UVec2::new(half_width, fixture.viewport_size.y));
    let right_half = RenderViewportRect::new(
        UVec2::new(half_width, 0),
        UVec2::new(
            fixture.viewport_size.x - half_width,
            fixture.viewport_size.y,
        ),
    );
    let mut extract = fixture.frame_extract(Vec::new());
    let left_camera = primary_surface_camera_descriptor(
        908,
        0,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        RenderLayerSet::default(),
        extract.view.camera.clone(),
    )
    .with_viewport(left_half);
    let right_camera = primary_surface_camera_descriptor(
        909,
        1,
        RenderCameraClear::Color(Vec4::new(0.0, 1.0, 0.0, 1.0)),
        RenderLayerSet::default(),
        extract.view.camera.clone(),
    )
    .with_viewport(right_half);
    extract.view = extract.view.with_cameras(vec![left_camera, right_camera]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("split-screen PrimarySurface frame should be capturable");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(frame.width, fixture.viewport_size.x);
    assert_eq!(frame.height, fixture.viewport_size.y);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let inset_margin = UVec2::new(8, 8);
    let inset_size = UVec2::new(half_width - 16, fixture.viewport_size.y - 16);
    let left_inset = RenderViewportRegion::new(inset_margin, inset_size);
    let right_inset = RenderViewportRegion::new(UVec2::new(half_width + 8, 8), inset_size);
    let inset_pixels = (inset_size.x * inset_size.y) as usize;
    let left_red = dominant_red_pixels_in_region(&frame, left_inset);
    let left_green = dominant_green_pixels_in_region(&frame, left_inset);
    let right_red = dominant_red_pixels_in_region(&frame, right_inset);
    let right_green = dominant_green_pixels_in_region(&frame, right_inset);

    assert!(
        left_red > inset_pixels * 9 / 10,
        "left Base camera should clear only the left viewport red; left_red={left_red}, left_green={left_green}, right_red={right_red}, right_green={right_green}, total={inset_pixels}"
    );
    assert!(
        left_green < inset_pixels / 20,
        "right Base camera green clear should not leak into the left viewport; green={left_green}, total={inset_pixels}"
    );
    assert!(
        right_green > inset_pixels * 9 / 10,
        "right Base camera should clear only the right viewport green; green={right_green}, total={inset_pixels}"
    );
    assert!(
        right_red < inset_pixels / 20,
        "left Base camera red clear should not leak into the right viewport; red={right_red}, total={inset_pixels}"
    );
}
