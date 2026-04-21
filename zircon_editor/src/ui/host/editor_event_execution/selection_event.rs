use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::EditorEventEffect;
use crate::core::editor_event::SelectionHostEvent;

use super::execution_outcome::ExecutionOutcome;
use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_selection(
    inner: &mut EditorEventRuntimeInner,
    event: &SelectionHostEvent,
) -> Result<ExecutionOutcome, String> {
    let changed = match event {
        SelectionHostEvent::SelectSceneNode { node_id } => inner
            .state
            .apply_intent(EditorIntent::SelectNode(*node_id))?,
    };
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}
