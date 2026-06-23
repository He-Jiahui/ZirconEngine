use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_route::HierarchyPointerRoute;

impl HierarchyPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HierarchyPointerRoute>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        Ok(self
            .route_intents
            .hierarchy_route_for_pointer_dispatch(&dispatch))
    }
}
