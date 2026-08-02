use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_dispatch::HierarchyPointerDispatch;
use super::hierarchy_pointer_route::HierarchyPointerRoute;

impl HierarchyPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        point: UiPoint,
    ) -> Result<HierarchyPointerDispatch, String> {
        self.refresh_row_metrics();
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        match route.as_ref() {
            Some(HierarchyPointerRoute::Node { item_index, .. }) => {
                self.state.hovered_item_index = Some(*item_index);
            }
            Some(HierarchyPointerRoute::ListSurface) | None => {
                self.state.hovered_item_index = None;
            }
        }

        Ok(HierarchyPointerDispatch {
            route,
            state: self.state.clone(),
        })
    }
}
