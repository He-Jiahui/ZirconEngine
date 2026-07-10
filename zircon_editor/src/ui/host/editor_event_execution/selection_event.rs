use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::EditorEventEffect;
use crate::core::editor_event::SelectionHostEvent;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_selection(
    shell: &mut WorkbenchShellStateData,
    event: &SelectionHostEvent,
) -> Result<ExecutionOutcome, String> {
    let changed = match event {
        SelectionHostEvent::SelectSceneNode { node_id } => shell
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
