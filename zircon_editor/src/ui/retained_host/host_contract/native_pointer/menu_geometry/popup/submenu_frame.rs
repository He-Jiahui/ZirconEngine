use super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::frames::{
    constrained_submenu_popup_frame, menu_popup_height, menu_popup_row_frame,
};

pub(super) fn nested_submenu_popup_frame(
    presentation: &HostWindowPresentationData,
    parent_popup: &FrameRect,
    selected_index: usize,
    child_count: usize,
    level: usize,
) -> FrameRect {
    let scroll_px = if level == 0 {
        presentation.menu_state.window_menu_scroll_px
    } else {
        0.0
    };
    let anchor = menu_popup_row_frame(parent_popup, selected_index, scroll_px);
    constrained_submenu_popup_frame(
        presentation,
        &anchor,
        parent_popup.width.max(1.0),
        menu_popup_height(child_count).max(1.0),
    )
}
