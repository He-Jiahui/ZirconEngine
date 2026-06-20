use super::super::super::super::data::FrameRect;
use super::metrics::{
    LIST_TOP, ROW_HEIGHT, ROW_INSET_X, ROW_SELECTED_MARK_WIDTH, ROW_TEXT_X, ROW_TEXT_Y,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_rect(
    panel_rect: &FrameRect,
    row: usize,
) -> FrameRect {
    FrameRect {
        x: panel_rect.x + ROW_INSET_X,
        y: panel_rect.y + LIST_TOP + row as f32 * ROW_HEIGHT,
        width: (panel_rect.width - ROW_INSET_X * 2.0).max(1.0),
        height: ROW_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_label_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: row_rect.x + ROW_TEXT_X,
        y: row_rect.y + ROW_TEXT_Y,
        width: (row_rect.width - ROW_TEXT_X * 2.0).max(1.0),
        height: (row_rect.height - ROW_TEXT_Y * 2.0).max(12.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_mark_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: row_rect.x,
        y: row_rect.y + 4.0,
        width: ROW_SELECTED_MARK_WIDTH,
        height: (row_rect.height - 8.0).max(1.0),
    }
}
