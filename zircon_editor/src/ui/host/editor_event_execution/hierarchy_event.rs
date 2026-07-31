use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::{EditorEventEffect, EditorHierarchyEvent};
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::execution_outcome::ExecutionOutcome;

pub(super) fn execute_hierarchy_event(
    shell: &mut WorkbenchShellStateData,
    event: &EditorHierarchyEvent,
) -> Result<ExecutionOutcome, String> {
    let changed = match event {
        EditorHierarchyEvent::ReparentNodes { node_ids, parent } => shell
            .state
            .apply_intent(EditorIntent::SetParents(node_ids.clone(), *parent))?,
        EditorHierarchyEvent::RenameNode { node_id, name } => shell
            .state
            .apply_intent(EditorIntent::RenameNode(*node_id, name.clone()))?,
    };
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}
