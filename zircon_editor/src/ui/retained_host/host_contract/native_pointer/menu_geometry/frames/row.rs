use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::menu_popup_metrics::{
    menu_popup_outer_padding, menu_popup_row_stride, MENU_POPUP_EDGE_INSET, MENU_POPUP_ROW_GAP,
    MENU_POPUP_ROW_HEIGHT,
};

pub(in crate::ui::retained_host::host_contract) fn scrolled_menu_frame(
    menu_frame: &FrameRect,
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    FrameRect {
        x: menu_frame.x - presentation.menu_state.menu_bar_scroll_px,
        y: menu_frame.y,
        width: menu_frame.width,
        height: menu_frame.height,
    }
}

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

pub(in crate::ui::retained_host::host_contract) fn menu_popup_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        menu_popup_outer_padding()
            + item_count as f32 * MENU_POPUP_ROW_HEIGHT
            + (item_count as f32 - 1.0) * MENU_POPUP_ROW_GAP
    }
}
