use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::layout::UiPoint;

use super::super::super::data::{FrameRect, PaneData};

pub(super) fn viewport_toolbar_route_hit(
    id: &str,
    x: f32,
    y: f32,
    surface_key: &str,
    pane: &PaneData,
    toolbar: &FrameRect,
) -> bool {
    let Some(surface_frame) = pane.viewport.toolbar_surface_frame.as_ref() else {
        return false;
    };
    let point = UiPoint::new(x - toolbar.x, y - toolbar.y);
    let Some(node_id) = hit_test_surface_frame(surface_frame, point).top_hit else {
        return false;
    };
    let Some(node) = surface_frame.arranged_tree.get(node_id) else {
        return false;
    };
    let Some(control_id) = node.control_id.as_deref() else {
        return false;
    };
    format!("viewport_toolbar_control.{surface_key}.{control_id}") == id
}
