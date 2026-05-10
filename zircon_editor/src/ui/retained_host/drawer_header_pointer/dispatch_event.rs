use zircon_runtime_interface::ui::dispatch::UiPointerEvent;

use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_target::HostDrawerHeaderPointerTarget;

impl HostDrawerHeaderPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HostDrawerHeaderPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
