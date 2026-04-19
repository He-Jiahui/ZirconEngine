use zircon_ui::dispatch::UiPointerEvent;

use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_target::WelcomeRecentPointerTarget;

impl WelcomeRecentPointerBridge {
    pub(in crate::ui::slint_host::welcome_recent_pointer) fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WelcomeRecentPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }
}
