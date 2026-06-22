use crate::ui::retained_host::hierarchy_pointer::constants::{
    ROW_GAP, ROW_HEIGHT, ROW_WIDTH_INSET, ROW_X, ROW_Y,
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
    Some(FrameRect {
        x: frame.x + ROW_X,
        y: frame.y + ROW_Y + row_index as f32 * (ROW_HEIGHT + ROW_GAP) - scroll_px.max(0.0),
        width: (frame.width - ROW_WIDTH_INSET).max(1.0),
        height: ROW_HEIGHT,
    })
}
