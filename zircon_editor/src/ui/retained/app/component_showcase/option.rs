use super::super::*;
use crate::ui::retained_host::app::showcase_event_inputs::select_option;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_component_showcase_option_selected(
        &mut self,
        control_id: &str,
        action_id: &str,
        option_id: &str,
    ) {
        self.focus_callback_source_window();
        if let Some(result) =
            self.dispatch_componentized_workbench_option_selected(control_id, action_id, option_id)
        {
            self.apply_dispatch_result(result);
            return;
        }
        let Some(binding_id) = self.component_showcase_binding_id_for_action(action_id) else {
            return;
        };
        self.dispatch_component_showcase_event(
            control_id,
            binding_id.as_str(),
            select_option(option_id, true),
        );
    }
}
