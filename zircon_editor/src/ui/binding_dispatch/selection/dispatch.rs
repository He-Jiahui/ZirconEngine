use crate::ui::{EditorUiBinding, EditorUiBindingPayload, SelectionCommand};

use super::super::error::EditorBindingDispatchError;
use super::selection_host_event::SelectionHostEvent;

pub fn dispatch_selection_binding(
    binding: &EditorUiBinding,
) -> Result<SelectionHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::SelectionCommand(SelectionCommand::SelectSceneNode { node_id }) => {
            Ok(SelectionHostEvent::SelectSceneNode { node_id: *node_id })
        }
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}
