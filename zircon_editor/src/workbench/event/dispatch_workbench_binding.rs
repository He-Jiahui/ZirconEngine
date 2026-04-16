use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};

use super::menu_action_from_id::menu_action_from_id;
use super::{WorkbenchHostEvent, WorkbenchHostEventError};

pub fn dispatch_workbench_binding(
    binding: &EditorUiBinding,
) -> Result<WorkbenchHostEvent, WorkbenchHostEventError> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => menu_action_from_id(action_id)
            .map(WorkbenchHostEvent::Menu)
            .ok_or_else(|| WorkbenchHostEventError::UnknownMenuAction(action_id.clone())),
        _ => Err(WorkbenchHostEventError::UnsupportedPayload),
    }
}
