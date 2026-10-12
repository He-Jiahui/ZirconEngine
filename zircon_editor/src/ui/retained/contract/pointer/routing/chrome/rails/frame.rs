use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::geometry::contains;

pub(super) fn activity_rail_frame_for_pointer(
    region: &FrameRect,
    rail_before_panel: bool,
    rail_width: f32,
    x: f32,
    y: f32,
) -> Option<FrameRect> {
    if !contains(region, x, y) || rail_width <= 0.0 {
        return None;
    }
    let rail_x = if rail_before_panel {
        region.x
    } else {
        region.x + (region.width - rail_width).max(0.0)
    };
    let rail = FrameRect {
        x: rail_x,
        y: region.y,
        width: rail_width.min(region.width.max(0.0)),
        height: region.height,
    };
    if contains(&rail, x, y) {
        Some(rail)
    } else {
        None
    }
}
