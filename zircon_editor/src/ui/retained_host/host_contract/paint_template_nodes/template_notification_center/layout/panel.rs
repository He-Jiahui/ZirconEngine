use super::super::super::super::data::FrameRect;
use super::metrics::{EMPTY_TEXT_TOP, HEADER_HEIGHT, HEADER_TOP, MESSAGE_HEIGHT, PANEL_PADDING_X};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn header_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    padded_panel_rect(panel_rect, HEADER_TOP, HEADER_HEIGHT)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    padded_panel_rect(panel_rect, EMPTY_TEXT_TOP, MESSAGE_HEIGHT)
}

fn padded_panel_rect(panel_rect: &FrameRect, y_offset: f32, height: f32) -> FrameRect {
    FrameRect {
        x: panel_rect.x + PANEL_PADDING_X,
        y: panel_rect.y + y_offset,
        width: (panel_rect.width - PANEL_PADDING_X * 2.0).max(1.0),
        height,
    }
}
