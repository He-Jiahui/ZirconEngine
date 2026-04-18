use zircon_ui::UiPointerEvent;

use super::workbench_host_page_pointer_bridge::WorkbenchHostPagePointerBridge;
use super::workbench_host_page_pointer_target::WorkbenchHostPagePointerTarget;

impl WorkbenchHostPagePointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WorkbenchHostPagePointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
