use crate::ui::{DraftCommand, EditorUiBinding, EditorUiBindingPayload};

use super::super::error::EditorBindingDispatchError;
use super::super::inspector::binding_value_to_string;
use super::draft_host_event::DraftHostEvent;

pub fn dispatch_draft_binding(
    binding: &EditorUiBinding,
) -> Result<DraftHostEvent, EditorBindingDispatchError> {
    let EditorUiBindingPayload::DraftCommand(command) = binding.payload() else {
        return Err(EditorBindingDispatchError::UnsupportedPayload);
    };

    match command {
        DraftCommand::SetInspectorField {
            subject_path,
            field_id,
            value,
        } => Ok(DraftHostEvent::SetInspectorField {
            subject_path: subject_path.clone(),
            field_id: field_id.clone(),
            value: binding_value_to_string(value, field_id)?,
        }),
        DraftCommand::SetMeshImportPath { value } => Ok(DraftHostEvent::SetMeshImportPath {
            value: value.clone(),
        }),
    }
}
