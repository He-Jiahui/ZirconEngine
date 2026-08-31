use super::RetainedEditorHost;
use crate::core::hub_link::HubFocusBindingTarget;

impl RetainedEditorHost {
    /// Synchronizes the owner-held focus inbox at the same commit boundary as the startup
    /// session projection. The binding itself performs no filesystem work when the committed
    /// `(project root, instance, generation)` identity is unchanged.
    pub(in crate::ui::retained_host::app) fn sync_hub_focus_binding(
        &mut self,
    ) -> Result<(), String> {
        let target = self
            .editor_manager
            .active_project_session_focus_target()
            .map(|(project_root, instance_id, session_generation)| {
                HubFocusBindingTarget::new(project_root, instance_id, session_generation)
            });
        self.hub_focus_binding
            .sync(target, &self.hub_focus_request_attention)
            .map_err(|error| format!("Hub focus binding transition failed: {error}"))
    }

    pub(in crate::ui::retained_host::app) fn acknowledge_hub_window_focus(
        &self,
    ) -> Result<(), String> {
        self.hub_focus_binding
            .acknowledge_native_window_focus()
            .map_err(|error| format!("Hub focus acknowledgement failed: {error}"))
    }

    pub(in crate::ui::retained_host::app) fn has_hub_focus_binding(&self) -> bool {
        self.hub_focus_binding.is_bound()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn retained_host_focus_binding_uses_the_committed_session_identity() {
        let source = include_str!("hub_focus_binding.rs");

        assert!(source.contains("active_project_session_focus_target()"));
        assert!(source.contains("HubFocusBindingTarget::new"));
        assert!(source.contains("hub_focus_binding\n            .sync"));
    }
}
