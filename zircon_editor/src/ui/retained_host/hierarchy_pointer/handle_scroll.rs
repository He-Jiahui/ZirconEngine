use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_dispatch::HierarchyPointerDispatch;
use super::hierarchy_pointer_route::HierarchyPointerRoute;

impl HierarchyPointerBridge {
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<HierarchyPointerDispatch, String> {
        self.refresh_row_metrics();
        let dispatched_route = self.dispatch_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )?;
        if dispatched_route.is_some() && delta.is_finite() {
            self.state.scroll_offset += delta;
            self.clamp_scroll_offset();
        }
        let route = self.project_route_at_point(dispatched_route, point);

        if let Some(HierarchyPointerRoute::Node { item_index, .. }) = route.as_ref() {
            self.state.hovered_item_index = Some(*item_index);
        }

        Ok(HierarchyPointerDispatch {
            route,
            state: self.state.clone(),
        })
    }
}
