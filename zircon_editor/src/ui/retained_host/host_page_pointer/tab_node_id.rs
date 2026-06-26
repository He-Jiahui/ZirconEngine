use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiRouteId};

use super::constants::{HOST_PAGE_ROUTE_ID_BASE, TAB_NODE_ID_BASE};

pub(super) fn tab_node_id(item_index: usize) -> UiNodeId {
    UiNodeId::new(TAB_NODE_ID_BASE + item_index as u64)
}

pub(super) fn tab_route_id(item_index: usize) -> UiRouteId {
    UiRouteId::new(HOST_PAGE_ROUTE_ID_BASE + TAB_NODE_ID_BASE + item_index as u64)
}

pub(super) fn overflow_route_id() -> UiRouteId {
    UiRouteId::new(HOST_PAGE_ROUTE_ID_BASE + 3)
}
