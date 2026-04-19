use zircon_runtime::ui::{event_ui::UiNodeId, event_ui::UiStateFlags, layout::UiFrame};

use super::constants::{
    BUTTON_HEIGHT, BUTTON_WIDTH, ITEM_GAP, ITEM_HEIGHT, ITEM_NODE_ID_BASE,
    OPEN_BUTTON_NODE_ID_BASE, OUTER_MARGIN, RECENT_PANEL_WIDTH, REMOVE_BUTTON_NODE_ID_BASE,
    VIEWPORT_HEIGHT_OFFSET, VIEWPORT_MIN_HEIGHT, VIEWPORT_X, VIEWPORT_Y,
};
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;

pub(in crate::ui::slint_host::welcome_recent_pointer) fn viewport_frame(
    layout: &WelcomeRecentPointerLayout,
) -> UiFrame {
    let outer_width = (layout.pane_size.width - OUTER_MARGIN * 2.0).max(0.0);
    let viewport_width = (RECENT_PANEL_WIDTH.min(outer_width) - 32.0).max(0.0);
    let viewport_height =
        (layout.pane_size.height - VIEWPORT_HEIGHT_OFFSET).max(VIEWPORT_MIN_HEIGHT);
    UiFrame::new(VIEWPORT_X, VIEWPORT_Y, viewport_width, viewport_height)
}

pub(in crate::ui::slint_host::welcome_recent_pointer) fn content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        item_count as f32 * ITEM_HEIGHT + (item_count as f32 - 1.0) * ITEM_GAP
    }
}

pub(in crate::ui::slint_host::welcome_recent_pointer) fn item_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(ITEM_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::slint_host::welcome_recent_pointer) fn open_button_node_id(
    index: usize,
) -> UiNodeId {
    UiNodeId::new(OPEN_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::slint_host::welcome_recent_pointer) fn remove_button_node_id(
    index: usize,
) -> UiNodeId {
    UiNodeId::new(REMOVE_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::slint_host::welcome_recent_pointer) fn button_size() -> (f32, f32) {
    (BUTTON_WIDTH, BUTTON_HEIGHT)
}

pub(in crate::ui::slint_host::welcome_recent_pointer) fn base_state(
    interactive: bool,
) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
