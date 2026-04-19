use zircon_runtime::ui::dispatch::UiPointerEvent;

use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_target::ViewportToolbarPointerTarget;

impl ViewportToolbarPointerBridge {
    pub(super) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<ViewportToolbarPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
