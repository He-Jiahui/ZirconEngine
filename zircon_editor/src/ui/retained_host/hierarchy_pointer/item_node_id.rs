use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiRouteId};

use super::constants::{HIERARCHY_ROUTE_ID_BASE, ITEM_NODE_ID_BASE, VIEWPORT_NODE_ID};

pub(super) fn item_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(ITEM_NODE_ID_BASE + index as u64)
}

pub(super) fn list_surface_route_id() -> UiRouteId {
    UiRouteId::new(HIERARCHY_ROUTE_ID_BASE + VIEWPORT_NODE_ID.0)
}

pub(super) fn item_route_id(index: usize) -> UiRouteId {
    UiRouteId::new(HIERARCHY_ROUTE_ID_BASE + ITEM_NODE_ID_BASE + index as u64)
}
