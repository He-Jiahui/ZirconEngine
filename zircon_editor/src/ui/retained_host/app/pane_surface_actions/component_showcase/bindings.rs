use super::super::*;
use super::MATERIAL_LAB_BINDING_PREFIX;
use crate::ui::template_runtime::SHOWCASE_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(super) fn component_showcase_binding_id_for_action(
        &mut self,
        action_id: &str,
    ) -> Option<String> {
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
    let mut output = String::with_capacity(value.len());
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.is_empty() && !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    if output.ends_with('_') {
        output.pop();
    }
    output
}

#[cfg(test)]
#[path = "bindings/camel_to_snake_tests.rs"]
mod camel_to_snake_tests;
