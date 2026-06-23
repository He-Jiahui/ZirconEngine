use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_route_intent::HostMenuPointerRouteIntent;

impl HostMenuPointerBridge {
    pub(in crate::ui::retained_host::menu_pointer) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HostMenuPointerRouteIntent>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(self
            .route_intents
            .menu_route_for_pointer_dispatch(&dispatch))
    }
}
