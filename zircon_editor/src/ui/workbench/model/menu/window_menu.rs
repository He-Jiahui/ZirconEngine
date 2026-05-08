use crate::core::editor_event::{MenuAction, ViewDescriptorId};
use crate::ui::workbench::event::menu_action_binding;

use super::super::menu_item_model::operation_path_for_menu_action;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_window_menu() -> MenuModel {
    let debug_observatory_action =
        MenuAction::OpenView(ViewDescriptorId::new("editor.debug_observatory"));
    MenuModel {
        label: "Window".to_string(),
        items: vec![
            MenuItemModel::leaf(
                "Debug Observatory",
                Some(debug_observatory_action.clone()),
                menu_action_binding(&debug_observatory_action),
                operation_path_for_menu_action(&debug_observatory_action),
                None,
                true,
            ),
            MenuItemModel::leaf(
                "Reset Layout",
                Some(MenuAction::ResetLayout),
                menu_action_binding(&MenuAction::ResetLayout),
                operation_path_for_menu_action(&MenuAction::ResetLayout),
                None,
                true,
            ),
        ],
    }
}
