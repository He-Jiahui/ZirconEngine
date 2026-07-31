use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiRouteId, UiStateFlags},
    layout::UiFrame,
};

use crate::ui::retained_host::welcome_recent_geometry::{
    welcome_recent_content_height, welcome_recent_viewport,
};

use super::constants::{
    ITEM_NODE_ID_BASE, OPEN_BUTTON_NODE_ID_BASE, REMOVE_BUTTON_NODE_ID_BASE, VIEWPORT_NODE_ID,
    WELCOME_RECENT_ROUTE_ID_BASE,
};
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;

pub(in crate::ui::retained_host::welcome_recent_pointer) fn viewport_frame(
    layout: &WelcomeRecentPointerLayout,
) -> UiFrame {
    welcome_recent_viewport(layout.pane_size)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn content_height(
    item_count: usize,
) -> f32 {
    welcome_recent_content_height(item_count)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn item_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(ITEM_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn open_button_node_id(
    index: usize,
) -> UiNodeId {
    UiNodeId::new(OPEN_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn remove_button_node_id(
    index: usize,
) -> UiNodeId {
    UiNodeId::new(REMOVE_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn list_surface_route_id() -> UiRouteId {
    UiRouteId::new(WELCOME_RECENT_ROUTE_ID_BASE + VIEWPORT_NODE_ID.0)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn item_route_id(
    index: usize,
) -> UiRouteId {
    UiRouteId::new(WELCOME_RECENT_ROUTE_ID_BASE + ITEM_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn open_button_route_id(
    index: usize,
) -> UiRouteId {
    UiRouteId::new(WELCOME_RECENT_ROUTE_ID_BASE + OPEN_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn remove_button_route_id(
    index: usize,
) -> UiRouteId {
    UiRouteId::new(WELCOME_RECENT_ROUTE_ID_BASE + REMOVE_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn base_state(
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
