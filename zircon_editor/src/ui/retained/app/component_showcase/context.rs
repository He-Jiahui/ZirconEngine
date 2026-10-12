use super::super::*;
use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_component_showcase_control_context_requested(
        &mut self,
        control_id: &str,
        action_id: &str,
        x: f64,
        y: f64,
    ) {
        self.focus_callback_source_window();
        let Some(mut binding_id) = self.component_showcase_binding_id_for_action(action_id) else {
            return;
        };
        if control_id == "ContextActionMenuDemo" && !binding_id.contains("ContextActionMenuOpenAt")
        {
            binding_id = "UiComponentShowcase/ContextActionMenuOpenAt".to_string();
        }
        self.dispatch_component_showcase_event(
            control_id,
            binding_id.as_str(),
            UiComponentShowcaseDemoEventInput::OpenPopupAt { x, y },
        );
    }
}
