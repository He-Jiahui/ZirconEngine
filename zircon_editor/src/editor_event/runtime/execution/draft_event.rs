use zircon_editor_ui::{DraftCommand, EditorUiBinding, EditorUiBindingPayload};
use zircon_ui::UiBindingValue;

use crate::apply_draft_binding;
use crate::editor_event::{EditorDraftEvent, EditorEventEffect};

use super::super::execution_outcome::ExecutionOutcome;
use super::super::runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_draft_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorDraftEvent,
) -> Result<ExecutionOutcome, String> {
    let binding = match event {
        EditorDraftEvent::SetInspectorField {
            subject_path,
            field_id,
            value,
        } => EditorUiBinding::new(
            "InspectorView",
            "DraftField",
            zircon_editor_ui::EditorUiEventKind::Change,
            EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                subject_path: subject_path.clone(),
                field_id: field_id.clone(),
                value: UiBindingValue::string(value.clone()),
            }),
        ),
        EditorDraftEvent::SetMeshImportPath { value } => EditorUiBinding::new(
            "AssetsView",
            "MeshImportPathEdited",
            zircon_editor_ui::EditorUiEventKind::Change,
            EditorUiBindingPayload::draft_command(DraftCommand::SetMeshImportPath {
                value: value.clone(),
            }),
        ),
    };

    let changed =
        apply_draft_binding(&mut inner.state, &binding).map_err(|error| error.to_string())?;
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}
