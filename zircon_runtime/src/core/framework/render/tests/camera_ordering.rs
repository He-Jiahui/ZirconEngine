use super::*;

#[test]
fn render_camera_order_report_carries_descriptor_render_type() {
    let mut overlay = CameraRenderDescriptor::from_camera_payload(Some(7), Default::default());
    overlay.render_type = CameraRenderType::Overlay;

    let report = sort_render_cameras([RenderCameraOrderInput::from_descriptor(7, overlay)]);

    assert_eq!(report.cameras.len(), 1);
    assert_eq!(report.cameras[0].render_type, CameraRenderType::Overlay);
}
