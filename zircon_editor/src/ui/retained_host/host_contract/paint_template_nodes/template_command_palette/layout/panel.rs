use super::super::super::super::data::FrameRect;
use super::metrics::{EMPTY_TEXT_Y, LINE_HEIGHT, PANEL_PADDING_X};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: panel_rect.x + PANEL_PADDING_X,
        y: panel_rect.y + EMPTY_TEXT_Y,
        width: (panel_rect.width - PANEL_PADDING_X * 2.0).max(1.0),
        height: LINE_HEIGHT,
    }
}
