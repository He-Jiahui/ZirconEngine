use crate::core::editor_event::SelectionHostEvent;
use crate::core::play::WorldDomain;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, SelectionCommand};

use super::super::error::EditorBindingDispatchError;

pub fn dispatch_selection_binding(
    binding: &EditorUiBinding,
) -> Result<SelectionHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::SelectionCommand(SelectionCommand::SelectSceneNode { node_id }) => {
            Ok(SelectionHostEvent::SelectSceneNode {
                world_domain: WorldDomain::Edit,
                node_id: *node_id,
            })
        }
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}
