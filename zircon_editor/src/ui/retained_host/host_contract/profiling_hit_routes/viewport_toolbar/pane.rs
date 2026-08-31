use super::super::super::data::{FrameRect, PaneData};
use super::super::geometry::contains;
use super::super::identity::profile_control_id;
use super::route_check::viewport_toolbar_route_hit;

pub(super) fn pane_route_hits_viewport_toolbar(
    id: &str,
    x: f32,
    y: f32,
    surface_key: &str,
    pane: &PaneData,
    content: &FrameRect,
) -> bool {
    let Some(expected_control_id) = profile_control_id(id, "viewport_toolbar_control", surface_key)
    else {
        return false;
    };
    if !matches!(pane.kind.as_str(), "Scene" | "Game")
        || !pane.show_toolbar
        || !contains(content, x, y)
    {
        return false;
    }
    let toolbar_height = 28.0_f32.min(content.height);
    let toolbar = FrameRect {
        x: content.x,
        y: content.y,
        width: content.width,
        height: toolbar_height,
    };
    viewport_toolbar_route_hit(expected_control_id, x, y, pane, &toolbar)
}
