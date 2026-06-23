use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiRouteId};

use super::constants::{
    ACTIVITY_RAIL_ROUTE_ID_BASE, LEFT_BUTTON_NODE_ID_BASE, LEFT_STRIP_NODE_ID,
    RIGHT_BUTTON_NODE_ID_BASE, RIGHT_STRIP_NODE_ID,
};
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;

pub(super) fn strip_button_node_id(side: HostActivityRailPointerSide, index: usize) -> UiNodeId {
    let base = match side {
        HostActivityRailPointerSide::Left => LEFT_BUTTON_NODE_ID_BASE,
        HostActivityRailPointerSide::Right => RIGHT_BUTTON_NODE_ID_BASE,
    };
    UiNodeId::new(base + index as u64)
}

pub(super) fn strip_route_id(side: HostActivityRailPointerSide) -> UiRouteId {
    let offset = match side {
        HostActivityRailPointerSide::Left => LEFT_STRIP_NODE_ID,
        HostActivityRailPointerSide::Right => RIGHT_STRIP_NODE_ID,
    };
    UiRouteId::new(ACTIVITY_RAIL_ROUTE_ID_BASE + offset.0)
}

pub(super) fn strip_button_route_id(side: HostActivityRailPointerSide, index: usize) -> UiRouteId {
    let base = match side {
        HostActivityRailPointerSide::Left => LEFT_BUTTON_NODE_ID_BASE,
        HostActivityRailPointerSide::Right => RIGHT_BUTTON_NODE_ID_BASE,
    };
    UiRouteId::new(ACTIVITY_RAIL_ROUTE_ID_BASE + base + index as u64)
}
