use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use super::constants::VIEWPORT_NODE_ID;
use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_dispatch::HierarchyPointerDispatch;
use super::hierarchy_pointer_target::HierarchyPointerTarget;
use super::to_public_route::to_public_route;

impl HierarchyPointerBridge {
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<HierarchyPointerDispatch, String> {
        let route = self.dispatch_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )?;

        if let Some(viewport) = self.surface.tree.node(VIEWPORT_NODE_ID) {
            let offset = viewport.scroll_state.unwrap_or_default().offset;
            if (self.state.scroll_offset - offset).abs() > f32::EPSILON {
                self.state.scroll_offset = offset;
                self.rebuild_surface();
            }
        }

        if let Some(HierarchyPointerTarget::Node { item_index, .. }) = route.as_ref() {
            self.state.hovered_item_index = Some(*item_index);
        }

        Ok(HierarchyPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }
}
