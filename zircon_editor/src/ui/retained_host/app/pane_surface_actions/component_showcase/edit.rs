use super::super::*;
use crate::ui::retained_host::app::showcase_event_inputs::demo_input_for_showcase_edit;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_component_showcase_control_edited(
        &mut self,
        control_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let Some(binding_id) = self.component_showcase_binding_id_for_action(action_id) else {
            return;
        };
        let input = demo_input_for_showcase_edit(binding_id.as_str(), value);
        self.dispatch_component_showcase_event(control_id, binding_id.as_str(), input);
    }
}
