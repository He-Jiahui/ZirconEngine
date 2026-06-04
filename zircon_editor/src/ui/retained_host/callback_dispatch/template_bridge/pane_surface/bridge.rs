use std::collections::BTreeMap;

use zircon_runtime_interface::ui::binding::UiEventKind;

use crate::ui::binding::EditorUiBinding;
use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_PANE_SURFACE_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostProjection};

#[cfg(test)]
use super::super::project_builtin_surface;
use super::super::{binding_for_control, project_builtin_surface_with_runtime};
use super::error::BuiltinPaneSurfaceTemplateBridgeError;

pub(crate) struct BuiltinPaneSurfaceTemplateBridge {
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: RetainedUiHostProjection,
}

impl BuiltinPaneSurfaceTemplateBridge {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, BuiltinPaneSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface(BUILTIN_PANE_SURFACE_DOCUMENT_ID)?;
        Ok(Self {
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn new_with_runtime(
        runtime: &EditorUiHostRuntime,
    ) -> Result<Self, BuiltinPaneSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface_with_runtime(runtime, BUILTIN_PANE_SURFACE_DOCUMENT_ID)?;
        Ok(Self {
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn binding_for_control(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Option<&EditorUiBinding> {
        binding_for_control(
            &self.bindings_by_id,
            &self.host_projection,
            control_id,
            event_kind,
        )
    }

    pub(crate) fn binding_by_id(&self, binding_id: &str) -> Option<&EditorUiBinding> {
        self.bindings_by_id.get(binding_id)
    }

    pub(crate) fn binding_id_for_action_id(&self, action_id: &str) -> Option<String> {
        if self.binding_by_id(action_id).is_some() {
            return Some(action_id.to_string());
        }
        self.host_projection
            .nodes
            .iter()
            .flat_map(|node| node.routes.iter())
            .find(|route| binding_path_action_id(&route.binding_id) == action_id)
            .map(|route| route.binding_id.clone())
    }
}

fn binding_path_action_id(binding_id: &str) -> String {
    binding_id
        .split(['/', '.', ':'])
        .filter(|segment| !segment.is_empty())
        .map(camel_to_snake_segment)
        .collect::<Vec<_>>()
        .join(".")
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
