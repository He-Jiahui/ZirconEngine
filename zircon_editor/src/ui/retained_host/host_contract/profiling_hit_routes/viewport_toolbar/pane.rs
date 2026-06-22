use super::super::super::data::{FrameRect, PaneData};
use super::super::geometry::contains;
use super::route_check::viewport_toolbar_route_hit;

pub(super) fn pane_route_hits_viewport_toolbar(
    id: &str,
    x: f32,
    y: f32,
    surface_key: &str,
    pane: &PaneData,
    content: &FrameRect,
) -> bool {
    let expected_prefix = format!("viewport_toolbar_control.{surface_key}.");
    if !id.starts_with(&expected_prefix)
        || !matches!(pane.kind.as_str(), "Scene" | "Game")
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
    viewport_toolbar_route_hit(id, x, y, surface_key, pane, &toolbar)
}
