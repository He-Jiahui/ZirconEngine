use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_target::HostActivityRailPointerTarget;

impl HostActivityRailPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HostActivityRailPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
