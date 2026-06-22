use crate::ui::retained_host::hierarchy_pointer::constants::{
    ROW_GAP, ROW_HEIGHT, ROW_WIDTH_INSET, ROW_X, ROW_Y,
};

use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn hierarchy_row_frame(viewport: &FrameRect, index: usize, scroll_px: f32) -> FrameRect {
    FrameRect {
        x: viewport.x + ROW_X,
        y: viewport.y + ROW_Y + index as f32 * (ROW_HEIGHT + ROW_GAP) - scroll_px,
        width: (viewport.width - ROW_WIDTH_INSET).max(0.0),
        height: ROW_HEIGHT,
    }
}
