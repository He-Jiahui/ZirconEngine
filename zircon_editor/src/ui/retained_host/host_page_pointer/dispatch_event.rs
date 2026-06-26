use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::error::HostPagePointerError;
use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_route::HostPagePointerRoute;

impl HostPagePointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HostPagePointerRoute>, HostPagePointerError> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)?;
        Ok(self
            .route_intents
            .host_page_route_for_pointer_dispatch(&dispatch))
    }
}
