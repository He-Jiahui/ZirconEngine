use super::super::super::super::super::data::FrameRect;
use super::super::super::layout::{WELCOME_ROW_GAP, WELCOME_ROW_HEIGHT};

pub(super) fn recent_project_row_frame(list: &FrameRect, index: usize) -> FrameRect {
    FrameRect {
        x: list.x + 8.0,
        y: list.y + 8.0 + index as f32 * (WELCOME_ROW_HEIGHT + WELCOME_ROW_GAP),
        width: (list.width - 16.0).max(0.0),
        height: WELCOME_ROW_HEIGHT,
    }
}
