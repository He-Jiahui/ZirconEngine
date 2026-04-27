use crate::core::editor_event::MenuAction;
use crate::ui::workbench::event::menu_action_binding;

use super::super::menu_item_model::operation_path_for_menu_action;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_window_menu() -> MenuModel {
    MenuModel {
        label: "Window".to_string(),
        items: vec![MenuItemModel {
            label: "Reset Layout".to_string(),
            action: Some(MenuAction::ResetLayout),
            binding: menu_action_binding(&MenuAction::ResetLayout),
            operation_path: operation_path_for_menu_action(&MenuAction::ResetLayout),
            shortcut: None,
            enabled: true,
        }],
    }
}
