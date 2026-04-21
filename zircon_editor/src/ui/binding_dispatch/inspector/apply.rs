use crate::ui::binding::EditorUiBinding;
use zircon_runtime::ui::binding::UiBindingValue;

use super::super::error::EditorBindingDispatchError;
use super::dispatch::dispatch_inspector_binding;
use super::field_value::{binding_value_to_string, parent_binding_value_to_string};
use super::subject_path::resolve_subject_path;
use crate::core::editing::intent::EditorIntent;
use crate::ui::workbench::state::EditorState;

pub fn apply_inspector_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<bool, EditorBindingDispatchError> {
    let batch = dispatch_inspector_binding(binding)?;
    let node_id = resolve_subject_path(state, &batch.subject_path)?;
    if state.viewport_controller.selected_node() != Some(node_id) {
        state
            .apply_intent(EditorIntent::SelectNode(node_id))
            .map_err(EditorBindingDispatchError::StateMutation)?;
    }

    for change in &batch.changes {
        apply_inspector_draft_field_value(state, &change.field_id, &change.value)?;
    }

    state
        .apply_intent(EditorIntent::ApplyInspectorChanges)
        .map_err(EditorBindingDispatchError::StateMutation)
}

pub(crate) fn apply_inspector_draft_field(
    state: &mut EditorState,
    subject_path: &str,
    field_id: &str,
    value: String,
) -> Result<bool, EditorBindingDispatchError> {
    let node_id = resolve_subject_path(state, subject_path)?;
    if state.viewport_controller.selected_node() != Some(node_id) {
        state
            .apply_intent(EditorIntent::SelectNode(node_id))
            .map_err(EditorBindingDispatchError::StateMutation)?;
    }

    apply_inspector_draft_field_value(state, field_id, &UiBindingValue::string(value))?;
    Ok(true)
}

fn apply_inspector_draft_field_value(
    state: &mut EditorState,
    field_id: &str,
    value: &UiBindingValue,
) -> Result<(), EditorBindingDispatchError> {
    match field_id {
        "name" => state.update_name_field(binding_value_to_string(value, field_id)?),
        "parent" => state.update_parent_field(parent_binding_value_to_string(value, field_id)?),
        "transform.translation.x" => {
            state.update_translation_field(0, binding_value_to_string(value, field_id)?);
        }
        "transform.translation.y" => {
            state.update_translation_field(1, binding_value_to_string(value, field_id)?);
        }
        "transform.translation.z" => {
            state.update_translation_field(2, binding_value_to_string(value, field_id)?);
        }
        other => {
            return Err(EditorBindingDispatchError::UnsupportedInspectorField(
                other.to_string(),
            ));
        }
    }

    Ok(())
}
