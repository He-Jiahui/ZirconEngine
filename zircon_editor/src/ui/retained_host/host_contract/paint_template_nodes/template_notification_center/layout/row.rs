use super::super::super::super::data::FrameRect;
use super::metrics::{
    MARK_HEIGHT, MARK_LEFT, MARK_TOP, MARK_WIDTH, MESSAGE_HEIGHT, MESSAGE_TOP, ROW_GAP, ROW_HEIGHT,
    ROW_INSET_X, ROW_TOP, TEXT_LEFT, TEXT_RIGHT_INSET, TITLE_HEIGHT, TITLE_TOP,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_rect(
    panel_rect: &FrameRect,
    row: usize,
) -> FrameRect {
    FrameRect {
        x: panel_rect.x + ROW_INSET_X,
        y: panel_rect.y + ROW_TOP + row as f32 * (ROW_HEIGHT + ROW_GAP),
        width: (panel_rect.width - ROW_INSET_X * 2.0).max(1.0),
        height: ROW_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn mark_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: row_rect.x + MARK_LEFT,
        y: row_rect.y + MARK_TOP,
        width: MARK_WIDTH,
        height: MARK_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn title_rect(
    row_rect: &FrameRect,
    width: f32,
) -> FrameRect {
    row_text_rect(row_rect, TITLE_TOP, width, TITLE_HEIGHT)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn message_rect(
    row_rect: &FrameRect,
    width: f32,
) -> FrameRect {
    row_text_rect(row_rect, MESSAGE_TOP, width, MESSAGE_HEIGHT)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_text_width(
    row_rect: &FrameRect,
) -> f32 {
    (row_rect.width - TEXT_LEFT - TEXT_RIGHT_INSET).max(1.0)
}

fn row_text_left(row_rect: &FrameRect) -> f32 {
    row_rect.x + TEXT_LEFT
}

fn row_text_rect(row_rect: &FrameRect, y_offset: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: row_text_left(row_rect),
        y: row_rect.y + y_offset,
        width,
        height,
    }
}
