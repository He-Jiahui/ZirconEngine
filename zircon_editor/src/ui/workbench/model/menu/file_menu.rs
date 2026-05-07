use crate::core::editor_event::MenuAction;
use crate::ui::workbench::event::menu_action_binding;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::menu_item_model::operation_path_for_menu_action;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_file_menu(chrome: &EditorChromeSnapshot) -> MenuModel {
    MenuModel {
        label: "File".to_string(),
        items: vec![
            MenuItemModel {
                label: "Open Project".to_string(),
                action: Some(MenuAction::OpenProject),
                binding: menu_action_binding(&MenuAction::OpenProject),
                operation_path: operation_path_for_menu_action(&MenuAction::OpenProject),
                shortcut: Some("Ctrl+O".to_string()),
                enabled: true,
                children: Vec::new(),
            },
            MenuItemModel {
                label: "Save Project".to_string(),
                action: Some(MenuAction::SaveProject),
                binding: menu_action_binding(&MenuAction::SaveProject),
                operation_path: operation_path_for_menu_action(&MenuAction::SaveProject),
                shortcut: Some("Ctrl+S".to_string()),
                enabled: chrome.project_open,
                children: Vec::new(),
            },
            MenuItemModel {
                label: "Save Layout".to_string(),
                action: Some(MenuAction::SaveLayout),
                binding: menu_action_binding(&MenuAction::SaveLayout),
                operation_path: operation_path_for_menu_action(&MenuAction::SaveLayout),
                shortcut: None,
                enabled: true,
                children: Vec::new(),
            },
            MenuItemModel {
                label: "Reset Layout".to_string(),
                action: Some(MenuAction::ResetLayout),
                binding: menu_action_binding(&MenuAction::ResetLayout),
                operation_path: operation_path_for_menu_action(&MenuAction::ResetLayout),
                shortcut: None,
                enabled: true,
                children: Vec::new(),
            },
        ],
    }
}
