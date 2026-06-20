use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_component_showcase_control_activated(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        let Some(binding_id) = self.component_showcase_binding_id_for_action(action_id) else {
            return;
        };
        let input = self.demo_input_for_showcase_action(control_id, binding_id.as_str());
        self.dispatch_component_showcase_event(control_id, binding_id.as_str(), input);
    }
}
