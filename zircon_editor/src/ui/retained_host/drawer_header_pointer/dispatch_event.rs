use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_route::HostDrawerHeaderPointerRoute;

impl HostDrawerHeaderPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HostDrawerHeaderPointerRoute>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(self
            .route_intents
            .drawer_header_route_for_pointer_dispatch(&dispatch))
    }
}
