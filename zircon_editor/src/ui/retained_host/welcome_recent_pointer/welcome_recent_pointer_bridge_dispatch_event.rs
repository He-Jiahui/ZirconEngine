use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_route_intent::WelcomeRecentPointerRouteIntent;

impl WelcomeRecentPointerBridge {
    pub(in crate::ui::retained_host::welcome_recent_pointer) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WelcomeRecentPointerRouteIntent>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(self
            .route_intents
            .welcome_recent_route_for_pointer_dispatch(&dispatch))
    }
}
