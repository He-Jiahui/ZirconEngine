use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use crate::scene::viewport::pointer::viewport_pointer_dispatch::ViewportPointerDispatch;

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

    fn handle_event(&mut self, event: UiPointerEvent) -> Result<ViewportPointerDispatch, String> {
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
        Ok(ViewportPointerDispatch { route })
    }
}
