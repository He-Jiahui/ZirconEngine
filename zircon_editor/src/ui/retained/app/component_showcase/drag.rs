use super::super::*;
use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_component_showcase_control_drag_delta(
        &mut self,
        control_id: &str,
        action_id: &str,
        delta: f64,
    ) {
        self.focus_callback_source_window();
        let Some(binding_id) = self.component_showcase_binding_id_for_action(action_id) else {
            return;
        };
        let input = if binding_id.contains("LargeDragUpdate") {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(delta)
        } else {
            UiComponentShowcaseDemoEventInput::DragDelta(delta)
        };
        self.dispatch_component_showcase_event(control_id, binding_id.as_str(), input);
    }
}
