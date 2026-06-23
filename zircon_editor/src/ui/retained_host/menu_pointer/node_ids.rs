use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiRouteId};

use super::constants::{
    DISMISS_NODE_ID, MENU_BUTTON_NODE_ID_BASE, MENU_ITEM_NODE_ID_BASE,
    MENU_ITEM_NODE_ID_LEVEL_STRIDE, MENU_ITEM_ROUTE_ID_LEVEL_STRIDE, MENU_ROUTE_ID_BASE,
    POPUP_NODE_ID, POPUP_NODE_ID_BASE,
};

pub(in crate::ui::retained_host::menu_pointer) fn menu_button_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(MENU_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::menu_pointer) fn popup_node_id(level: usize) -> UiNodeId {
    if level == 0 {
        POPUP_NODE_ID
    } else {
        UiNodeId::new(POPUP_NODE_ID_BASE + level as u64)
    }
}

pub(in crate::ui::retained_host::menu_pointer) fn menu_item_node_id(
    level: usize,
    index: usize,
) -> UiNodeId {
    UiNodeId::new(
        MENU_ITEM_NODE_ID_BASE + level as u64 * MENU_ITEM_NODE_ID_LEVEL_STRIDE + index as u64,
    )
}

pub(in crate::ui::retained_host::menu_pointer) fn menu_button_route_id(index: usize) -> UiRouteId {
    UiRouteId::new(MENU_ROUTE_ID_BASE + MENU_BUTTON_NODE_ID_BASE + index as u64)
}

pub(in crate::ui::retained_host::menu_pointer) fn dismiss_route_id() -> UiRouteId {
    UiRouteId::new(MENU_ROUTE_ID_BASE + DISMISS_NODE_ID.0)
}

pub(in crate::ui::retained_host::menu_pointer) fn popup_route_id(level: usize) -> UiRouteId {
    UiRouteId::new(MENU_ROUTE_ID_BASE + POPUP_NODE_ID.0 + level as u64)
}

pub(in crate::ui::retained_host::menu_pointer) fn menu_item_route_id(
    level: usize,
    index: usize,
) -> UiRouteId {
    UiRouteId::new(
        MENU_ROUTE_ID_BASE
            + MENU_ITEM_NODE_ID_BASE
            + level as u64 * MENU_ITEM_ROUTE_ID_LEVEL_STRIDE
            + index as u64,
    )
}
