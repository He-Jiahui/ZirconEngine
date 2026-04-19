use zircon_ui::dispatch::UiPointerEvent;

use super::workbench_menu_pointer_bridge::WorkbenchMenuPointerBridge;
use super::workbench_menu_pointer_target::WorkbenchMenuPointerTarget;

impl WorkbenchMenuPointerBridge {
    pub(in crate::ui::slint_host::menu_pointer) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WorkbenchMenuPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
