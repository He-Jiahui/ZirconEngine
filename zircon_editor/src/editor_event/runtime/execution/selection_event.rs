use crate::EditorIntent;
use crate::SelectionHostEvent;

use super::super::execution_outcome::ExecutionOutcome;
use super::super::runtime_inner::EditorEventRuntimeInner;

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
            crate::editor_event::EditorEventEffect::PresentationChanged,
            crate::editor_event::EditorEventEffect::ReflectionChanged,
        ],
    })
}
