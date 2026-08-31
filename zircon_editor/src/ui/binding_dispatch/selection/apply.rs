use crate::core::editor_event::SelectionHostEvent;
use crate::ui::binding::EditorUiBinding;
use crate::ui::workbench::state::EditorState;

use super::super::error::EditorBindingDispatchError;
use super::dispatch::dispatch_selection_binding;

pub fn apply_selection_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<bool, EditorBindingDispatchError> {
    match dispatch_selection_binding(binding)? {
        SelectionHostEvent::SelectSceneNode {
            world_domain,
            node_id,
        } => state
            .select_node_in_world(world_domain, node_id)
            .map_err(EditorBindingDispatchError::State),
    }
}
