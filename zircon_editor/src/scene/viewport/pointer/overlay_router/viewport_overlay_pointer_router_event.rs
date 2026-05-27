use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use crate::scene::viewport::pointer::{
    runtime_picking_adapter::runtime_pointer_input_for_event,
    viewport_pointer_dispatch::ViewportPointerDispatch,
};

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<ViewportPointerDispatch, String> {
        self.handle_event(UiPointerEvent::new(UiPointerEventKind::Move, point))
    }

    pub(crate) fn handle_down(
        &mut self,
        point: UiPoint,
    ) -> Result<ViewportPointerDispatch, String> {
        self.handle_event(UiPointerEvent::new(UiPointerEventKind::Down, point))
    }

    #[cfg(test)]
    pub(crate) fn handle_up(&mut self, point: UiPoint) -> Result<ViewportPointerDispatch, String> {
        self.handle_event(UiPointerEvent::new(UiPointerEventKind::Up, point))
    }

    #[cfg(test)]
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        scroll_delta: f32,
    ) -> Result<ViewportPointerDispatch, String> {
        self.handle_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(scroll_delta),
        )
    }

    fn handle_event(&mut self, event: UiPointerEvent) -> Result<ViewportPointerDispatch, String> {
        let point = event.point;
        let runtime_input = runtime_pointer_input_for_event(&event);
        if let Ok(mut shared) = self.shared.lock() {
            shared.last_route = None;
        }
        self.surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let route = self
            .shared
            .lock()
            .map_err(|_| "viewport pointer shared resolution lock poisoned".to_string())?
            .last_route
            .clone();
        Ok(ViewportPointerDispatch {
            route,
            runtime_input: Some(runtime_input),
            picking_debug_feed: Some(self.debug_feed_at(point)?),
        })
    }
}
