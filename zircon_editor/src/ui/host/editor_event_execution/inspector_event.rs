use crate::core::editor_event::EditorInspectorEvent;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::binding_dispatch::apply_inspector_binding;

use super::common::scene_effects;
use super::execution_outcome::ExecutionOutcome;
use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_inspector_event(
    inner: &mut EditorEventRuntimeInner,
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
        apply_inspector_binding(&mut inner.state, &binding).map_err(|error| error.to_string())?;
    Ok(ExecutionOutcome {
        changed,
        effects: scene_effects(),
    })
}
