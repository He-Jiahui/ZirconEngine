use crate::ui::EditorUiBinding;

use super::super::error::EditorBindingDispatchError;
use super::super::inspector::apply_inspector_draft_field;
use super::dispatch::dispatch_draft_binding;
use super::draft_host_event::DraftHostEvent;
use crate::EditorState;

pub fn apply_draft_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<bool, EditorBindingDispatchError> {
    match dispatch_draft_binding(binding)? {
        DraftHostEvent::SetInspectorField {
            subject_path,
            field_id,
            value,
        } => apply_inspector_draft_field(state, &subject_path, &field_id, value),
        DraftHostEvent::SetMeshImportPath { value } => {
            state.set_mesh_import_path(value);
            Ok(true)
        }
    }
}
