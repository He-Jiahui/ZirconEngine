use crate::ui::EditorUiBinding;

use super::super::error::EditorBindingDispatchError;
use super::dispatch::dispatch_selection_binding;
use super::selection_host_event::SelectionHostEvent;
use crate::{EditorIntent, EditorState};

pub fn apply_selection_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<bool, EditorBindingDispatchError> {
    match dispatch_selection_binding(binding)? {
        SelectionHostEvent::SelectSceneNode { node_id } => state
            .apply_intent(EditorIntent::SelectNode(node_id))
            .map_err(EditorBindingDispatchError::StateMutation),
    }
}
