use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};

use crate::apply_inspector_binding;
use crate::editor_event::EditorInspectorEvent;

use super::super::execution_outcome::ExecutionOutcome;
use super::super::runtime_inner::EditorEventRuntimeInner;
use super::common::scene_effects;

pub(super) fn execute_inspector_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorInspectorEvent,
) -> Result<ExecutionOutcome, String> {
    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        zircon_editor_ui::EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            event.subject_path.clone(),
            event.changes.clone(),
        ),
    );
    let changed =
        apply_inspector_binding(&mut inner.state, &binding).map_err(|error| error.to_string())?;
    Ok(ExecutionOutcome {
        changed,
        effects: scene_effects(),
    })
}
