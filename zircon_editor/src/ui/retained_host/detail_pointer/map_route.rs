use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::bridge_constants::VIEWPORT_NODE_ID;
use super::scroll_surface_pointer_route::ScrollSurfacePointerRoute;

pub(super) fn map_route(target: Option<UiNodeId>) -> Option<ScrollSurfacePointerRoute> {
    match target {
        Some(VIEWPORT_NODE_ID) => Some(ScrollSurfacePointerRoute::Viewport),
        _ => None,
    }
}
