use crate::ui::binding::EditorUiBinding;
use zircon_runtime_interface::ui::binding::UiBindingValue;

use super::super::error::EditorBindingDispatchError;
use super::dispatch::dispatch_inspector_binding;
use super::field_value::{binding_value_to_string, parent_binding_value_to_string};
use super::subject_path::resolve_subject_path;
use crate::core::editing::intent::EditorIntent;
use crate::core::play::WorldDomain;
use crate::ui::workbench::state::EditorState;

pub fn apply_inspector_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<bool, EditorBindingDispatchError> {
    let batch = dispatch_inspector_binding(binding)?;
    let node_id = resolve_subject_path(state, &batch.subject_path)?;
    state.ensure_inspector_binding_can_begin()?;
    if matches!(
        state.viewport_controller.selection().active_domain(),
        WorldDomain::Play(_)
    ) {
        if state.viewport_controller.selection().active_primary() != Some(node_id) {
            return Err(EditorBindingDispatchError::InvalidSubjectPath(
                batch.subject_path,
            ));
        }
        let changes = batch
            .changes
            .iter()
            .map(|change| {
                let value = if change.field_id == "parent" {
                    parent_binding_value_to_string(&change.value, &change.field_id)
                } else {
                    binding_value_to_string(&change.value, &change.field_id)
                }?;
                Ok((change.field_id.clone(), value))
            })
            .collect::<Result<Vec<_>, EditorBindingDispatchError>>()?;
        return state
            .apply_play_inspector_changes(node_id, &changes)
            .map_err(EditorBindingDispatchError::State);
    }
    let checkpoint = state.inspector_binding_ui_checkpoint()?;
    let result = (|| {
        state.ensure_transaction_context_selection_is_current()?;

        if state.viewport_controller.selection().active_primary() != Some(node_id) {
            state
                .apply_intent(EditorIntent::SelectNode(node_id))
                .map_err(EditorBindingDispatchError::State)?;
        }

        for change in &batch.changes {
            apply_inspector_draft_field_value(state, &change.field_id, &change.value)?;
        }

        state
            .apply_intent(EditorIntent::ApplyInspectorChanges)
            .map_err(EditorBindingDispatchError::State)
    })();

    match result {
        Ok(changed) => Ok(changed),
        Err(error) => match state.restore_inspector_binding_ui_checkpoint(checkpoint) {
            Ok(()) => Err(error),
            Err(rollback) => Err(EditorBindingDispatchError::InspectorBindingRollback {
                cause: Box::new(error),
                rollback,
            }),
        },
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
            .map_err(EditorBindingDispatchError::State)?;
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
            if !state.can_edit_dynamic_component_field(other)? {
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
