use crate::core::editor_event::MenuAction;
use zircon_runtime::scene::components::NodeKind;

use crate::ui::workbench::event::menu_action_binding;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::menu_item_model::operation_path_for_menu_action;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_selection_menu(chrome: &EditorChromeSnapshot) -> MenuModel {
    MenuModel {
        label: "Selection".to_string(),
        items: vec![
            MenuItemModel {
                label: "Create Cube".to_string(),
                action: Some(MenuAction::CreateNode(NodeKind::Cube)),
                binding: menu_action_binding(&MenuAction::CreateNode(NodeKind::Cube)),
                operation_path: operation_path_for_menu_action(&MenuAction::CreateNode(
                    NodeKind::Cube,
                )),
                shortcut: None,
                enabled: true,
            },
            MenuItemModel {
                label: "Create Camera".to_string(),
                action: Some(MenuAction::CreateNode(NodeKind::Camera)),
                binding: menu_action_binding(&MenuAction::CreateNode(NodeKind::Camera)),
                operation_path: operation_path_for_menu_action(&MenuAction::CreateNode(
                    NodeKind::Camera,
                )),
                shortcut: None,
                enabled: true,
            },
            MenuItemModel {
                label: "Create Directional Light".to_string(),
                action: Some(MenuAction::CreateNode(NodeKind::DirectionalLight)),
                binding: menu_action_binding(&MenuAction::CreateNode(NodeKind::DirectionalLight)),
                operation_path: operation_path_for_menu_action(&MenuAction::CreateNode(
                    NodeKind::DirectionalLight,
                )),
                shortcut: None,
                enabled: true,
            },
            MenuItemModel {
                label: "Delete Selection".to_string(),
                action: Some(MenuAction::DeleteSelected),
                binding: menu_action_binding(&MenuAction::DeleteSelected),
                operation_path: operation_path_for_menu_action(&MenuAction::DeleteSelected),
                shortcut: Some("Delete".to_string()),
                enabled: chrome.inspector.is_some(),
            },
        ],
    }
}
