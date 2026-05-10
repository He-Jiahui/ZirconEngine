use zircon_runtime::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::bridge_constants::VIEWPORT_NODE_ID;
use super::map_route::map_route;
use super::scroll_surface_pointer_bridge::ScrollSurfacePointerBridge;
use super::scroll_surface_pointer_dispatch::ScrollSurfacePointerDispatch;

impl ScrollSurfacePointerBridge {
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<ScrollSurfacePointerDispatch, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(
                &self.dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
            )
            .map_err(|error| error.to_string())?;
        if let Some(viewport) = self.surface.tree.node(VIEWPORT_NODE_ID) {
            let offset = viewport.scroll_state.unwrap_or_default().offset;
            if (self.state.scroll_offset - offset).abs() > f32::EPSILON {
                self.state.scroll_offset = offset;
                self.rebuild_surface();
            }
        }

        Ok(ScrollSurfacePointerDispatch {
            route: map_route(dispatch.handled_by.or(dispatch.route.target)),
            state: self.state.clone(),
        })
    }
}
