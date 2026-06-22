use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

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
        x: popup.x + 6.0,
        y: popup.y + 6.0 + row as f32 * 30.0 - scroll_px,
        width: (popup.width - 12.0).max(0.0),
        height: 28.0,
    }
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        12.0 + item_count as f32 * 28.0 + (item_count as f32 - 1.0) * 2.0
    }
}
