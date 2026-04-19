use zircon_runtime::ui::dispatch::UiPointerEvent;

use super::workbench_activity_rail_pointer_bridge::WorkbenchActivityRailPointerBridge;
use super::workbench_activity_rail_pointer_target::WorkbenchActivityRailPointerTarget;

impl WorkbenchActivityRailPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WorkbenchActivityRailPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
