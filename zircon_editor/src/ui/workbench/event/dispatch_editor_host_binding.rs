use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};

use super::menu_action_from_id::menu_action_from_id;
use super::{EditorHostEvent, EditorHostEventError};

pub fn dispatch_editor_host_binding(
    binding: &EditorUiBinding,
) -> Result<EditorHostEvent, EditorHostEventError> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => menu_action_from_id(action_id)
            .map(EditorHostEvent::Menu)
            .ok_or_else(|| EditorHostEventError::UnknownMenuAction(action_id.clone())),
        _ => Err(EditorHostEventError::UnsupportedPayload),
    }
}
