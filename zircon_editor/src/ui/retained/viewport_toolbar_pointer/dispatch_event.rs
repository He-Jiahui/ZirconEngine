use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

impl ViewportToolbarPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<ViewportToolbarPointerRoute>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(self
            .route_intents
            .viewport_toolbar_route_for_pointer_dispatch(&dispatch))
    }
}
