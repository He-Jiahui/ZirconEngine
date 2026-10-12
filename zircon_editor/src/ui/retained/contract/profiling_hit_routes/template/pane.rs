use super::super::super::data::{FrameRect, PaneData};
use super::super::geometry::contains;
use super::super::identity::profile_control_id;
use super::route_check::surface_frame_route_hit;

pub(super) fn pane_route_hits_template(
    id: &str,
    x: f32,
    y: f32,
    surface: &str,
    pane: &PaneData,
    content: &FrameRect,
) -> bool {
    let Some(expected_control_id) = profile_control_id(id, "template", surface) else {
        return false;
    };
    if !contains(content, x, y) {
        return false;
    }
    let mut body = content.clone();
    if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar_height = 28.0_f32.min(content.height);
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    let Some(surface_frame) = pane.body_surface_frame.as_ref() else {
        return false;
    };
    surface_frame_route_hit(expected_control_id, x, y, surface_frame, &body)
}
