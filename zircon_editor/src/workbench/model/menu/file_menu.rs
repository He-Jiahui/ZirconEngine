use crate::snapshot::EditorChromeSnapshot;
use crate::workbench::event::menu_action_binding;

use super::super::menu_action::MenuAction;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_file_menu(chrome: &EditorChromeSnapshot) -> MenuModel {
    MenuModel {
        label: "File".to_string(),
        items: vec![
            MenuItemModel {
                label: "Open Project".to_string(),
                action: MenuAction::OpenProject,
                binding: menu_action_binding(&MenuAction::OpenProject),
                shortcut: Some("Ctrl+O".to_string()),
                enabled: true,
            },
            MenuItemModel {
                label: "Save Project".to_string(),
                action: MenuAction::SaveProject,
                binding: menu_action_binding(&MenuAction::SaveProject),
                shortcut: Some("Ctrl+S".to_string()),
                enabled: chrome.project_open,
            },
            MenuItemModel {
                label: "Save Layout".to_string(),
                action: MenuAction::SaveLayout,
                binding: menu_action_binding(&MenuAction::SaveLayout),
                shortcut: None,
                enabled: true,
            },
            MenuItemModel {
                label: "Reset Layout".to_string(),
                action: MenuAction::ResetLayout,
                binding: menu_action_binding(&MenuAction::ResetLayout),
                shortcut: None,
                enabled: true,
            },
        ],
    }
}
