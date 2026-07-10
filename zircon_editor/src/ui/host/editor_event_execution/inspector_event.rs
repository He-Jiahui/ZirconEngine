use crate::core::editor_event::EditorInspectorEvent;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::binding_dispatch::apply_inspector_binding;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::common::scene_effects;
use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_inspector_event(
    shell: &mut WorkbenchShellStateData,
    event: &EditorInspectorEvent,
) -> Result<ExecutionOutcome, String> {
    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            event.subject_path.clone(),
            event.changes.clone(),
        ),
    );
    let changed =
        apply_inspector_binding(&mut shell.state, &binding).map_err(|error| error.to_string())?;
    Ok(ExecutionOutcome {
        changed,
        effects: scene_effects(),
    })
}
