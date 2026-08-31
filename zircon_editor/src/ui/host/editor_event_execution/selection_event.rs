use crate::core::editor_event::EditorEventEffect;
use crate::core::editor_event::SelectionHostEvent;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::state::EditorStateOperationError;

use super::common::effects_when;
use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_selection(
    shell: &mut WorkbenchShellStateData,
    event: &SelectionHostEvent,
) -> Result<ExecutionOutcome, EditorStateOperationError> {
    let changed = match event {
        SelectionHostEvent::SelectSceneNode {
            world_domain,
            node_id,
        } => shell.state.select_node_in_world(*world_domain, *node_id)?,
    };
    Ok(ExecutionOutcome {
        changed,
        effects: effects_when(
            changed,
            [
                EditorEventEffect::PresentationChanged,
                EditorEventEffect::ReflectionChanged,
            ],
        ),
    })
}
