use zircon_runtime_interface::ui::event_ui::UiRouteId;

use super::constants::{HIERARCHY_ROUTE_ID_BASE, VIEWPORT_NODE_ID};

pub(super) fn list_surface_route_id() -> UiRouteId {
    UiRouteId::new(HIERARCHY_ROUTE_ID_BASE + VIEWPORT_NODE_ID.0)
}
