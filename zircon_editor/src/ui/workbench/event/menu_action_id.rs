use crate::core::editor_event::MenuAction;

use super::node_kind_id::node_kind_id;

pub(super) fn menu_action_id(action: &MenuAction) -> String {
    match action {
        MenuAction::OpenProject => "OpenProject".to_string(),
        MenuAction::OpenScene => "OpenScene".to_string(),
        MenuAction::CreateScene => "CreateScene".to_string(),
        MenuAction::SaveProject => "SaveProject".to_string(),
        MenuAction::SaveLayout => "SaveLayout".to_string(),
        MenuAction::ResetLayout => "ResetLayout".to_string(),
        MenuAction::Undo => "Undo".to_string(),
        MenuAction::Redo => "Redo".to_string(),
        MenuAction::CreateNode(kind) => format!("CreateNode.{}", node_kind_id(kind)),
        MenuAction::DeleteSelected => "DeleteSelected".to_string(),
        MenuAction::OpenView(descriptor_id) => format!("OpenView.{}", descriptor_id.0),
    }
}
