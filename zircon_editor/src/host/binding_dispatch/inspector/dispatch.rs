use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};

use super::super::error::EditorBindingDispatchError;
use super::inspector_binding_batch::InspectorBindingBatch;

pub fn dispatch_inspector_binding(
    binding: &EditorUiBinding,
) -> Result<InspectorBindingBatch, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::InspectorFieldBatch {
            subject_path,
            changes,
        } => Ok(InspectorBindingBatch {
            subject_path: subject_path.clone(),
            changes: changes.clone(),
        }),
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}
