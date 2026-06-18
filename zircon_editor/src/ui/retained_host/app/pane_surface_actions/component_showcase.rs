use super::*;
use crate::ui::retained_host::app::showcase_event_inputs::{
    demo_input_for_showcase_action, demo_input_for_showcase_edit, select_option,
};
use crate::ui::template_runtime::builtin::MATERIAL_COMPONENT_LAB_WINDOW_DOCUMENT_ID;
use crate::ui::template_runtime::{UiComponentShowcaseDemoEventInput, SHOWCASE_DOCUMENT_ID};

const MATERIAL_LAB_BINDING_PREFIX: &str = "MaterialLab/";

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

    fn component_showcase_binding_id_for_action(&mut self, action_id: &str) -> Option<String> {
        if action_id.starts_with(MATERIAL_LAB_BINDING_PREFIX) {
            return Some(action_id.to_string());
        }
        if let Err(error) = self.ensure_component_showcase_runtime_loaded() {
            self.set_status_line(error);
            return None;
        }
        let binding_id = self
            .component_showcase_runtime
            .project_document(SHOWCASE_DOCUMENT_ID)
            .ok()
            .and_then(|projection| {
                projection.bindings.into_iter().find_map(|binding| {
                    if binding.binding_id == action_id
                        || component_showcase_action_id_for_binding_id(&binding.binding_id)
                            == action_id
                    {
                        Some(binding.binding_id)
                    } else {
                        None
                    }
                })
            });
        if binding_id.is_none() {
            self.set_status_line(format!("Unknown component showcase action {action_id}"));
        }
        binding_id
    }

    fn dispatch_component_showcase_event(
        &mut self,
        control_id: &str,
        action_id: &str,
        input: UiComponentShowcaseDemoEventInput,
    ) {
        if action_id.starts_with(MATERIAL_LAB_BINDING_PREFIX) {
            self.dispatch_material_lab_event(control_id, action_id);
            return;
        }
        if let Err(error) = self.ensure_component_showcase_runtime_loaded() {
            self.set_status_line(error);
            return;
        }

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
            Ok(result) => {
                self.set_status_line(
                    result
                        .status_text
                        .unwrap_or_else(|| format!("Showcase event dispatched: {control_id}")),
                );
                if result.changed || result.refresh_projection {
                    self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                }
            }
            Err(error) => {
                self.set_status_line(format!("Showcase event failed: {error}"));
            }
        }
    }

    fn dispatch_material_lab_event(&mut self, control_id: &str, action_id: &str) {
        if let Err(error) = self.ensure_component_showcase_runtime_loaded() {
            self.set_status_line(error);
            return;
        }

        let binding = self
            .component_showcase_runtime
            .project_document(MATERIAL_COMPONENT_LAB_WINDOW_DOCUMENT_ID)
            .ok()
            .and_then(|projection| {
                projection
                    .bindings
                    .into_iter()
                    .find(|binding| binding.binding_id == action_id)
            });
        if binding.is_none() {
            self.set_status_line(format!("Unknown Material Lab action {action_id}"));
            return;
        }

        self.set_status_line(format!(
            "Material Lab feedback: {control_id} -> {}",
            action_id.replace('/', ".")
        ));
    }

    fn demo_input_for_showcase_action(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) -> UiComponentShowcaseDemoEventInput {
        if let Some(payload) = self.take_active_reference_drag_payload_for_drop(action_id) {
            return UiComponentShowcaseDemoEventInput::DropReference { payload };
        }
        if action_id.contains("VirtualListScrolled") {
            return self.next_showcase_virtual_list_range(control_id);
        }
        if action_id.contains("PagedListNextPage") {
            return self.next_showcase_page(control_id);
        }
        demo_input_for_showcase_action(control_id, action_id)
    }

    fn next_showcase_virtual_list_range(
        &self,
        control_id: &str,
    ) -> UiComponentShowcaseDemoEventInput {
        let current_start = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "viewport_start")
            .unwrap_or(0);
        let visible_count = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "viewport_count")
            .unwrap_or(25)
            .max(1);
        let total_count = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "total_count")
            .unwrap_or(current_start + visible_count)
            .max(0);
        let max_start = total_count.saturating_sub(visible_count).max(0);
        let start = (current_start + visible_count).min(max_start);
        UiComponentShowcaseDemoEventInput::SetVisibleRange {
            start,
            count: visible_count,
        }
    }

    fn next_showcase_page(&self, control_id: &str) -> UiComponentShowcaseDemoEventInput {
        let page_index = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "page_index")
            .unwrap_or(0);
        let page_size = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "page_size")
            .unwrap_or(100)
            .max(1);
        let page_count = self
            .component_showcase_runtime
            .showcase_demo_value_i64(control_id, "page_count")
            .unwrap_or(page_index + 2)
            .max(1);
        UiComponentShowcaseDemoEventInput::SetPage {
            page_index: (page_index + 1).min(page_count - 1),
            page_size,
        }
    }
}

fn component_showcase_action_id_for_binding_id(binding_id: &str) -> String {
    let Some(suffix) = binding_id.strip_prefix("UiComponentShowcase/") else {
        return binding_id
            .split(['/', '.', ':'])
            .filter(|segment| !segment.is_empty())
            .map(camel_to_snake_segment)
            .collect::<Vec<_>>()
            .join(".");
    };
    format!("ui_component_showcase.{}", camel_to_snake_segment(suffix))
}

fn camel_to_snake_segment(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}
