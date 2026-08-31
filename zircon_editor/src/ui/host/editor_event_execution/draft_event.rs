use zircon_runtime_interface::ui::binding::UiBindingValue;

use crate::core::editor_event::{EditorDraftEvent, EditorEventEffect};
use crate::ui::binding::{
    DraftCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use crate::ui::binding_dispatch::apply_draft_binding;
use crate::ui::binding_dispatch::EditorBindingDispatchError;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_draft_event(
    shell: &mut WorkbenchShellStateData,
    event: &EditorDraftEvent,
) -> Result<ExecutionOutcome, EditorBindingDispatchError> {
    let binding = match event {
        EditorDraftEvent::SetInspectorField {
            subject_path,
            field_id,
            value,
        } => EditorUiBinding::new(
            "InspectorView",
            "DraftField",
            EditorUiEventKind::Change,
            EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
                subject_path: subject_path.clone(),
                field_id: field_id.clone(),
                value: UiBindingValue::string(value.clone()),
            }),
        ),
        EditorDraftEvent::SetMeshImportPath { value } => EditorUiBinding::new(
            "AssetsView",
            "MeshImportPathEdited",
            EditorUiEventKind::Change,
            EditorUiBindingPayload::draft_command(DraftCommand::SetMeshImportPath {
                value: value.clone(),
            }),
        ),
    };

    let changed = apply_draft_binding(&mut shell.state, &binding)?;
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}
