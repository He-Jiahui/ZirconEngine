use crate::view::ViewDescriptorId;

use super::super::model::MenuAction;
use super::node_kind_from_id::node_kind_from_id;

pub(super) fn menu_action_from_id(action_id: &str) -> Option<MenuAction> {
    match action_id {
        "OpenProject" => Some(MenuAction::OpenProject),
        "OpenScene" => Some(MenuAction::OpenScene),
        "CreateScene" => Some(MenuAction::CreateScene),
        "SaveProject" => Some(MenuAction::SaveProject),
        "SaveLayout" => Some(MenuAction::SaveLayout),
        "ResetLayout" => Some(MenuAction::ResetLayout),
        "Undo" => Some(MenuAction::Undo),
        "Redo" => Some(MenuAction::Redo),
        "DeleteSelected" => Some(MenuAction::DeleteSelected),
        _ => {
            if let Some(kind) = action_id.strip_prefix("CreateNode.") {
                return node_kind_from_id(kind).map(MenuAction::CreateNode);
            }
            if let Some(descriptor_id) = action_id.strip_prefix("OpenView.") {
                return Some(MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)));
            }
            None
        }
    }
}
