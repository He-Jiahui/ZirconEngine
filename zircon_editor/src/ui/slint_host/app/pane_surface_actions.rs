use super::showcase_event_inputs::{
    demo_input_for_showcase_action, demo_input_for_showcase_edit, select_option,
};
use super::*;
use crate::ui::template_runtime::{UiComponentShowcaseDemoEventInput, SHOWCASE_DOCUMENT_ID};

impl SlintEditorHost {
    pub(super) fn dispatch_pane_surface_control_clicked(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        if control_id == "ModulePluginAction" {
            self.dispatch_module_plugin_action(action_id);
            return;
        }
        let Some(result) = callback_dispatch::dispatch_builtin_pane_surface_control(
            &self.runtime,
            &self.pane_surface_bridge,
            control_id,
            UiEventKind::Click,
            vec![UiBindingValue::string(action_id)],
        ) else {
            self.set_status_line(format!("Unknown pane surface control {control_id}"));
            return;
        };

        self.apply_dispatch_result(result);
    }

    pub(super) fn dispatch_component_showcase_control_activated(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        let input = self.demo_input_for_showcase_action(control_id, action_id);
        self.dispatch_component_showcase_event(control_id, action_id, input);
    }

    pub(super) fn dispatch_component_showcase_control_drag_delta(
        &mut self,
        control_id: &str,
        action_id: &str,
        delta: f64,
    ) {
        self.focus_callback_source_window();
        let input = if action_id.contains("LargeDragUpdate") {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(delta)
        } else {
            UiComponentShowcaseDemoEventInput::DragDelta(delta)
        };
        self.dispatch_component_showcase_event(control_id, action_id, input);
    }

    pub(super) fn dispatch_component_showcase_control_edited(
        &mut self,
        control_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let input = demo_input_for_showcase_edit(action_id, value);
        self.dispatch_component_showcase_event(control_id, action_id, input);
    }

    pub(super) fn dispatch_component_showcase_control_context_requested(
        &mut self,
        control_id: &str,
        action_id: &str,
        x: f64,
        y: f64,
    ) {
        self.focus_callback_source_window();
        let action_id = if control_id == "ContextActionMenuDemo"
            && !action_id.contains("ContextActionMenuOpenAt")
        {
            "UiComponentShowcase/ContextActionMenuOpenAt"
        } else {
            action_id
        };
        self.dispatch_component_showcase_event(
            control_id,
            action_id,
            UiComponentShowcaseDemoEventInput::OpenPopupAt { x, y },
        );
    }

    pub(super) fn dispatch_component_showcase_option_selected(
        &mut self,
        control_id: &str,
        action_id: &str,
        option_id: &str,
    ) {
        self.focus_callback_source_window();
        self.dispatch_component_showcase_event(
            control_id,
            action_id,
            select_option(option_id, true),
        );
    }

    fn dispatch_component_showcase_event(
        &mut self,
        control_id: &str,
        action_id: &str,
        input: UiComponentShowcaseDemoEventInput,
    ) {
        let binding = self
            .component_showcase_runtime
            .project_document(SHOWCASE_DOCUMENT_ID)
            .ok()
            .and_then(|projection| {
                projection
                    .bindings
                    .into_iter()
                    .find(|binding| binding.binding_id == action_id)
            });
        let Some(binding) = binding else {
            self.set_status_line(format!("Unknown component showcase action {action_id}"));
            return;
        };

        match self
            .component_showcase_runtime
            .apply_showcase_demo_binding(&binding.binding, input)
        {
            Ok(()) => {
                self.set_status_line(format!("Showcase event dispatched: {control_id}"));
                self.presentation_dirty = true;
            }
            Err(error) => {
                self.set_status_line(format!("Showcase event failed: {error}"));
            }
        }
    }

    fn demo_input_for_showcase_action(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) -> UiComponentShowcaseDemoEventInput {
        if let Some(payload) = self.take_active_reference_drag_payload_for_drop(action_id) {
            return UiComponentShowcaseDemoEventInput::DropReference { payload };
        }
        demo_input_for_showcase_action(control_id, action_id)
    }
}
