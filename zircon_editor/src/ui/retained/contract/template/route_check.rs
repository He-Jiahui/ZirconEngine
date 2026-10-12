use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::{layout::UiPoint, surface::UiSurfaceFrame};

use super::super::super::data::FrameRect;

pub(super) fn surface_frame_route_hit(
    expected_control_id: &str,
    x: f32,
    y: f32,
    surface_frame: &UiSurfaceFrame,
    body: &FrameRect,
) -> bool {
    let point = UiPoint::new(x - body.x, y - body.y);
    let Some(node_id) = hit_test_surface_frame(surface_frame, point).top_hit else {
        return false;
    };
    let Some(node) = surface_frame.arranged_tree.get(node_id) else {
        return false;
    };
    node.control_id.as_deref() == Some(expected_control_id)
}
