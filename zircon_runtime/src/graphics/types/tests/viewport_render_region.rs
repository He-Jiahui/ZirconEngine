use crate::core::framework::render::{
    CameraRenderDescriptor, RenderViewportRect, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;

use super::ViewportRenderRegion;

#[test]
fn viewport_region_defaults_to_full_target_without_camera_rect() {
    let region = ViewportRenderRegion::full_target(UVec2::new(640, 360));

    assert_eq!(region.physical_position(), UVec2::ZERO);
    assert_eq!(region.physical_size(), UVec2::new(640, 360));
    assert!(!region.is_empty());
}

#[test]
fn viewport_region_clamps_camera_rect_to_target() {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    let mut viewport = RenderViewportRect::new(UVec2::new(600, 300), UVec2::new(128, 128));
    viewport.depth_min = -1.0;
    viewport.depth_max = 2.0;
    camera.viewport_rect = Some(viewport);

    let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

    assert_eq!(region.physical_position(), UVec2::new(600, 300));
    assert_eq!(region.physical_size(), UVec2::new(40, 60));
    assert_eq!(region.depth_min, 0.0);
    assert_eq!(region.depth_max, 1.0);
    assert!(!region.is_empty());
}

#[test]
fn viewport_region_clamps_fully_outside_rect_to_last_in_bounds_pixel() {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(1280, 720),
        UVec2::new(320, 180),
    ));

    let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

    assert_eq!(region.physical_position(), UVec2::new(639, 359));
    assert_eq!(region.physical_size(), UVec2::new(1, 1));
    assert!(!region.is_empty());
}

#[test]
fn viewport_region_maps_local_postprocess_coords_to_physical_target_coords() {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(320, 0),
        UVec2::new(320, 180),
    ));

    let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

    assert_eq!(region.physical_origin(), [320, 0]);
    assert_eq!(
        region.local_to_physical_coord(UVec2::new(0, 0)),
        UVec2::new(320, 0)
    );
    assert_eq!(
        region.local_to_physical_coord(UVec2::new(319, 179)),
        UVec2::new(639, 179)
    );
    assert_eq!(
        region.local_to_physical_coord(UVec2::new(999, 999)),
        UVec2::new(639, 179)
    );
}

#[test]
fn viewport_region_reports_local_rect_for_graph_owned_targets() {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(320, 0),
        UVec2::new(320, 180),
    ));

    let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

    assert_eq!(region.local_position(), UVec2::ZERO);
    assert_eq!(region.local_size(), UVec2::new(320, 180));
    assert_eq!(region.physical_position(), UVec2::new(320, 0));
}

#[test]
fn viewport_region_preserves_output_rect_when_local_render_size_changes() {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(80, 40),
        UVec2::new(160, 120),
    ));

    let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(320, 240))
        .with_local_size(UVec2::new(80, 60));

    assert_eq!(region.physical_position(), UVec2::new(80, 40));
    assert_eq!(region.physical_size(), UVec2::new(160, 120));
    assert_eq!(region.local_position(), UVec2::ZERO);
    assert_eq!(region.local_size(), UVec2::new(80, 60));
    assert_eq!(
        region.local_to_physical_coord(UVec2::new(79, 59)),
        UVec2::new(239, 159)
    );
}

#[test]
fn viewport_region_derives_origin_zero_region_for_graph_owned_targets() {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.viewport_rect = Some(RenderViewportRect::new(
        UVec2::new(80, 40),
        UVec2::new(160, 120),
    ));

    let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(320, 240))
        .with_local_size(UVec2::new(80, 60))
        .local_render_region();

    assert_eq!(region.physical_position(), UVec2::ZERO);
    assert_eq!(region.physical_size(), UVec2::new(80, 60));
    assert_eq!(region.local_size(), UVec2::new(80, 60));
}
