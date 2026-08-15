use crate::ui::binding::EditorUiBinding;
use zircon_runtime_interface::ui::binding::UiBindingValue;

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
    state
        .ensure_inspector_binding_can_begin()
        .map_err(EditorBindingDispatchError::StateMutation)?;
    let checkpoint = state
        .inspector_binding_ui_checkpoint()
        .map_err(EditorBindingDispatchError::StateMutation)?;
    let result = (|| {
        state
            .ensure_transaction_context_selection_is_current()
            .map_err(EditorBindingDispatchError::StateMutation)?;

        if state.viewport_controller.selection().active_primary() != Some(node_id) {
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
    })();

    match result {
        Ok(changed) => Ok(changed),
        Err(error) => {
            state
                .restore_inspector_binding_ui_checkpoint(checkpoint)
                .map_err(|rollback| {
                    EditorBindingDispatchError::StateMutation(format!(
                        "{error}; inspector binding rollback failed: {rollback}"
                    ))
                })?;
            Err(error)
        }
    }
}

pub(crate) fn apply_inspector_draft_field(
    state: &mut EditorState,
    subject_path: &str,
    field_id: &str,
    value: String,
) -> Result<bool, EditorBindingDispatchError> {
    let node_id = resolve_subject_path(state, subject_path)?;
    if state.viewport_controller.selection().active_primary() != Some(node_id) {
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
        "transform.scale.x" => {
            state.update_scale_field(0, binding_value_to_string(value, field_id)?);
        }
        "transform.scale.y" => {
            state.update_scale_field(1, binding_value_to_string(value, field_id)?);
        }
        "transform.scale.z" => {
            state.update_scale_field(2, binding_value_to_string(value, field_id)?);
        }
        other => {
            if !state.can_edit_dynamic_component_field(other) {
                return Err(EditorBindingDispatchError::UnsupportedInspectorField(
                    other.to_string(),
                ));
            }
            state.update_dynamic_component_field(
                other.to_string(),
                binding_value_to_string(value, field_id)?,
            );
        }
    }

    Ok(())
}
