use zircon_runtime_interface::ui::layout::UiPoint;

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_dispatch::HierarchyPointerDispatch;
use super::hierarchy_pointer_route::HierarchyPointerRoute;

impl HierarchyPointerBridge {
    pub(crate) fn handle_scroll(&mut self, point: UiPoint, delta: f32) -> HierarchyPointerDispatch {
        self.refresh_row_metrics();
        let pointer_is_inside = self.route_at_point(point).is_some();
        if pointer_is_inside && delta.is_finite() {
            self.state.scroll_offset += delta;
            self.clamp_scroll_offset();
        }
        let route = self.route_at_point(point);

        if let Some(HierarchyPointerRoute::Node { item_index }) = route {
            self.state.hovered_item_index = Some(item_index);
        }

        HierarchyPointerDispatch {
            route,
            state: self.state,
        }
    }
}
