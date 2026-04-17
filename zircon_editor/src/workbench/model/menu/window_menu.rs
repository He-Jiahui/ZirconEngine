use crate::workbench::event::menu_action_binding;

use super::super::menu_action::MenuAction;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_window_menu() -> MenuModel {
    MenuModel {
        label: "Window".to_string(),
        items: vec![MenuItemModel {
            label: "Reset Layout".to_string(),
            action: MenuAction::ResetLayout,
            binding: menu_action_binding(&MenuAction::ResetLayout),
            shortcut: None,
            enabled: true,
        }],
    }
}
