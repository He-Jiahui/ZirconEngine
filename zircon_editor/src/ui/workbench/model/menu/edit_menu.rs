use crate::core::editor_event::MenuAction;
use crate::ui::workbench::event::menu_action_binding;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::menu_item_model::operation_path_for_menu_action;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_edit_menu(chrome: &EditorChromeSnapshot) -> MenuModel {
    MenuModel {
        label: "Edit".to_string(),
        items: vec![
            MenuItemModel {
                label: "Undo".to_string(),
                action: Some(MenuAction::Undo),
                binding: menu_action_binding(&MenuAction::Undo),
                operation_path: operation_path_for_menu_action(&MenuAction::Undo),
                shortcut: Some("Ctrl+Z".to_string()),
                enabled: chrome.can_undo,
                children: Vec::new(),
            },
            MenuItemModel {
                label: "Redo".to_string(),
                action: Some(MenuAction::Redo),
                binding: menu_action_binding(&MenuAction::Redo),
                operation_path: operation_path_for_menu_action(&MenuAction::Redo),
                shortcut: Some("Ctrl+Shift+Z".to_string()),
                enabled: chrome.can_redo,
                children: Vec::new(),
            },
        ],
    }
}
