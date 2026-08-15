use super::super::super::super::data::FrameRect;
use super::super::super::super::menu_popup_metrics::{
    menu_popup_outer_padding, menu_popup_row_stride, MENU_POPUP_EDGE_INSET, MENU_POPUP_ROW_HEIGHT,
};

pub(in crate::ui::retained_host::host_contract) fn menu_popup_row_frame(
    popup: &FrameRect,
    row: usize,
    scroll_px: f32,
) -> FrameRect {
    FrameRect {
        x: popup.x + MENU_POPUP_EDGE_INSET,
        y: popup.y + MENU_POPUP_EDGE_INSET + row as f32 * menu_popup_row_stride() - scroll_px,
        width: (popup.width - menu_popup_outer_padding()).max(0.0),
        height: MENU_POPUP_ROW_HEIGHT,
    }
}
