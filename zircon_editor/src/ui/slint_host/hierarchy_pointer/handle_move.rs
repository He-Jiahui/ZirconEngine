use zircon_ui::{dispatch::UiPointerEvent, UiPoint, UiPointerEventKind};

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_dispatch::HierarchyPointerDispatch;
use super::hierarchy_pointer_target::HierarchyPointerTarget;
use super::to_public_route::to_public_route;

impl HierarchyPointerBridge {
    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<HierarchyPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        match route.as_ref() {
            Some(HierarchyPointerTarget::Node { item_index, .. }) => {
                self.state.hovered_item_index = Some(*item_index);
            }
            Some(HierarchyPointerTarget::ListSurface) | None => {
                self.state.hovered_item_index = None;
            }
        }

        Ok(HierarchyPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }
}
