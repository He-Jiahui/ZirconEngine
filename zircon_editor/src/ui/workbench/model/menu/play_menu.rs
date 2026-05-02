use crate::core::editor_event::MenuAction;
use crate::ui::workbench::event::menu_action_binding;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::startup::EditorSessionMode;

use super::super::menu_item_model::operation_path_for_menu_action;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_play_menu(chrome: &EditorChromeSnapshot) -> MenuModel {
    let playing = chrome.session_mode == EditorSessionMode::Playing;
    MenuModel {
        label: "Play".to_string(),
        items: vec![
            MenuItemModel {
                label: "Enter Play Mode".to_string(),
                action: Some(MenuAction::EnterPlayMode),
                binding: menu_action_binding(&MenuAction::EnterPlayMode),
                operation_path: operation_path_for_menu_action(&MenuAction::EnterPlayMode),
                shortcut: Some("F5".to_string()),
                enabled: chrome.project_open && !playing,
            },
            MenuItemModel {
                label: "Exit Play Mode".to_string(),
                action: Some(MenuAction::ExitPlayMode),
                binding: menu_action_binding(&MenuAction::ExitPlayMode),
                operation_path: operation_path_for_menu_action(&MenuAction::ExitPlayMode),
                shortcut: Some("Shift+F5".to_string()),
                enabled: playing,
            },
        ],
    }
}
