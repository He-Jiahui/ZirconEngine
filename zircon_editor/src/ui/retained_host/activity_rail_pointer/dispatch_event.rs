use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_route::HostActivityRailPointerRoute;

impl HostActivityRailPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HostActivityRailPointerRoute>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(self
            .route_intents
            .activity_rail_route_for_pointer_dispatch(&dispatch))
    }
}
