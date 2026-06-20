use super::super::super::super::data::FrameRect;
use super::metrics::{
    LINE_HEIGHT, PANEL_PADDING_X, SEARCH_HEIGHT, SEARCH_TEXT_X, SEARCH_TEXT_Y, SEARCH_TOP,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: panel_rect.x + PANEL_PADDING_X,
        y: panel_rect.y + SEARCH_TOP,
        width: (panel_rect.width - PANEL_PADDING_X * 2.0).max(1.0),
        height: SEARCH_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_text_rect(
    search_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: search_rect.x + SEARCH_TEXT_X,
        y: search_rect.y + SEARCH_TEXT_Y,
        width: (search_rect.width - SEARCH_TEXT_X * 2.0).max(1.0),
        height: LINE_HEIGHT,
    }
}
