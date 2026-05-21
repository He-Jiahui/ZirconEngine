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
            create_node_item("Create Cube", NodeKind::Cube),
            create_node_item("Create Camera", NodeKind::Camera),
            create_node_item("Create Ambient Light", NodeKind::AmbientLight),
            create_node_item("Create Directional Light", NodeKind::DirectionalLight),
            create_node_item("Create Point Light", NodeKind::PointLight),
            create_node_item("Create Rect Light", NodeKind::RectLight),
            create_node_item("Create Spot Light", NodeKind::SpotLight),
            MenuItemModel {
                label: "Delete Selection".to_string(),
                action: Some(MenuAction::DeleteSelected),
                binding: menu_action_binding(&MenuAction::DeleteSelected),
                operation_path: operation_path_for_menu_action(&MenuAction::DeleteSelected),
                shortcut: Some("Delete".to_string()),
                enabled: chrome.inspector.is_some(),
                children: Vec::new(),
            },
        ],
    }
}

fn create_node_item(label: &str, kind: NodeKind) -> MenuItemModel {
    let action = MenuAction::CreateNode(kind);
    MenuItemModel {
        label: label.to_string(),
        action: Some(action.clone()),
        binding: menu_action_binding(&action),
        operation_path: operation_path_for_menu_action(&action),
        shortcut: None,
        enabled: true,
        children: Vec::new(),
    }
}
