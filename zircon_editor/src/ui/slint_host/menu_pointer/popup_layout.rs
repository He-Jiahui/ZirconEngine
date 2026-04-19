use zircon_runtime::ui::layout::UiFrame;

use super::constants::{
    POPUP_PADDING, POPUP_ROW_GAP, POPUP_ROW_HEIGHT, POPUP_WIDTHS, WINDOW_MENU_INDEX,
};
use super::menu_items_for_layout::menu_items_for_layout;
use super::workbench_menu_pointer_layout::WorkbenchMenuPointerLayout;

pub(in crate::ui::slint_host::menu_pointer) fn popup_frame(
    layout: &WorkbenchMenuPointerLayout,
    menu_index: usize,
) -> UiFrame {
    let button_frame = layout.button_frames[menu_index];
    let popup_y = button_frame.y + button_frame.height + 3.0;
    let width = POPUP_WIDTHS[menu_index];
    let height = if menu_index == WINDOW_MENU_INDEX {
        layout.window_popup_height.max(72.0)
    } else {
        popup_content_height(menu_items_for_layout(layout, menu_index).len()) + POPUP_PADDING * 2.0
    };
    UiFrame::new(button_frame.x, popup_y, width, height)
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_content_frame(
    popup_frame: UiFrame,
) -> UiFrame {
    UiFrame::new(
        popup_frame.x + POPUP_PADDING,
        popup_frame.y + POPUP_PADDING,
        (popup_frame.width - POPUP_PADDING * 2.0).max(0.0),
        (popup_frame.height - POPUP_PADDING * 2.0).max(0.0),
    )
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        item_count as f32 * POPUP_ROW_HEIGHT + (item_count as f32 - 1.0) * POPUP_ROW_GAP
    }
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_viewport_extent(
    layout: &WorkbenchMenuPointerLayout,
    menu_index: usize,
) -> f32 {
    (popup_frame(layout, menu_index).height - POPUP_PADDING * 2.0).max(0.0)
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_scroll_metrics(
    layout: &WorkbenchMenuPointerLayout,
    menu_index: usize,
) -> (f32, f32) {
    let viewport_extent = popup_viewport_extent(layout, menu_index);
    let content_extent = popup_content_height(menu_items_for_layout(layout, menu_index).len());
    (viewport_extent, content_extent)
}
