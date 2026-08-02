use crate::ui::retained_host::hierarchy_pointer::{
    current_hierarchy_row_metrics, hierarchy_row_width, hierarchy_row_y,
};
use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn hierarchy_row_damage(
    frame: &FrameRect,
    row_index: i32,
    scroll_px: f32,
) -> Option<FrameRect> {
    if row_index < 0 {
        return None;
    }
    let metrics = current_hierarchy_row_metrics();
    Some(FrameRect {
        x: frame.x + metrics.row_x,
        y: frame.y + hierarchy_row_y(metrics, row_index as usize, scroll_px),
        width: hierarchy_row_width(frame.width, metrics).max(1.0),
        height: metrics.row_height,
    })
}
