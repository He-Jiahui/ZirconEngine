use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind, tree::UiTreeError,
};

use crate::scene::viewport::pointer::{
    precision::lock_shared_resolution_state, viewport_pointer_dispatch::ViewportPointerDispatch,
};

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<ViewportPointerDispatch, UiTreeError> {
        self.handle_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
    }

    pub(crate) fn handle_down(
        &mut self,
        point: UiPoint,
    ) -> Result<ViewportPointerDispatch, UiTreeError> {
        self.handle_event(UiPointerEvent::new(UiPointerEventKind::Down, point))
    }

    #[cfg(test)]
    pub(crate) fn handle_up(
        &mut self,
        point: UiPoint,
    ) -> Result<ViewportPointerDispatch, UiTreeError> {
        self.handle_event(UiPointerEvent::new(UiPointerEventKind::Up, point))
    }

    #[cfg(test)]
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        scroll_delta: f32,
    ) -> Result<ViewportPointerDispatch, UiTreeError> {
        self.handle_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(scroll_delta),
        )
    }

    fn handle_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<ViewportPointerDispatch, UiTreeError> {
        {
            let mut shared = lock_shared_resolution_state(self.shared.as_ref());
            shared.last_route = None;
            shared.last_debug_feed = None;
        }
        self.surface
            .dispatch_pointer_event(&self.dispatcher, event)?;
        let shared = lock_shared_resolution_state(self.shared.as_ref());
        Ok(ViewportPointerDispatch {
            route: shared.last_route.clone(),
            picking_debug_feed: shared.last_debug_feed.clone(),
        })
    }
}
