use zircon_runtime_interface::ui::layout::UiPoint;

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_dispatch::HierarchyPointerDispatch;
use super::hierarchy_pointer_route::HierarchyPointerRoute;

impl HierarchyPointerBridge {
    pub(crate) fn handle_click(&mut self, point: UiPoint) -> HierarchyPointerDispatch {
        self.refresh_row_metrics();
        let route = self.route_at_point(point);
        match route {
            Some(HierarchyPointerRoute::Node { item_index }) => {
                self.state.hovered_item_index = Some(item_index);
            }
            Some(HierarchyPointerRoute::ListSurface) | None => {
                self.state.hovered_item_index = None;
            }
        }

        HierarchyPointerDispatch {
            route,
            state: self.state,
        }
    }
}
