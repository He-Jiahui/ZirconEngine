use zircon_scene::NodeKind;

use crate::snapshot::EditorChromeSnapshot;
use crate::view::ViewDescriptorId;
use crate::workbench::event::menu_action_binding;

use super::menu_action::MenuAction;
use super::menu_bar_model::MenuBarModel;
use super::menu_item_model::MenuItemModel;
use super::menu_model::MenuModel;

pub(super) fn default_menu_bar(chrome: &EditorChromeSnapshot) -> MenuBarModel {
    MenuBarModel {
        menus: vec![
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
            },
            MenuModel {
                label: "Edit".to_string(),
                items: vec![
                    MenuItemModel {
                        label: "Undo".to_string(),
                        action: MenuAction::Undo,
                        binding: menu_action_binding(&MenuAction::Undo),
                        shortcut: Some("Ctrl+Z".to_string()),
                        enabled: chrome.can_undo,
                    },
                    MenuItemModel {
                        label: "Redo".to_string(),
                        action: MenuAction::Redo,
                        binding: menu_action_binding(&MenuAction::Redo),
                        shortcut: Some("Ctrl+Shift+Z".to_string()),
                        enabled: chrome.can_redo,
                    },
                ],
            },
            MenuModel {
                label: "Selection".to_string(),
                items: vec![
                    MenuItemModel {
                        label: "Create Cube".to_string(),
                        action: MenuAction::CreateNode(NodeKind::Cube),
                        binding: menu_action_binding(&MenuAction::CreateNode(NodeKind::Cube)),
                        shortcut: None,
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Create Camera".to_string(),
                        action: MenuAction::CreateNode(NodeKind::Camera),
                        binding: menu_action_binding(&MenuAction::CreateNode(NodeKind::Camera)),
                        shortcut: None,
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Create Directional Light".to_string(),
                        action: MenuAction::CreateNode(NodeKind::DirectionalLight),
                        binding: menu_action_binding(&MenuAction::CreateNode(
                            NodeKind::DirectionalLight,
                        )),
                        shortcut: None,
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Delete Selection".to_string(),
                        action: MenuAction::DeleteSelected,
                        binding: menu_action_binding(&MenuAction::DeleteSelected),
                        shortcut: Some("Delete".to_string()),
                        enabled: chrome.inspector.is_some(),
                    },
                ],
            },
            MenuModel {
                label: "View".to_string(),
                items: builtin_view_menu_items(),
            },
            MenuModel {
                label: "Window".to_string(),
                items: vec![MenuItemModel {
                    label: "Reset Layout".to_string(),
                    action: MenuAction::ResetLayout,
                    binding: menu_action_binding(&MenuAction::ResetLayout),
                    shortcut: None,
                    enabled: true,
                }],
            },
            MenuModel {
                label: "Help".to_string(),
                items: vec![MenuItemModel {
                    label: "Workbench Guide".to_string(),
                    action: MenuAction::OpenView(ViewDescriptorId::new("editor.asset_browser")),
                    binding: menu_action_binding(&MenuAction::OpenView(ViewDescriptorId::new(
                        "editor.asset_browser",
                    ))),
                    shortcut: None,
                    enabled: true,
                }],
            },
        ],
    }
}

fn builtin_view_menu_items() -> Vec<MenuItemModel> {
    [
        ("Project", "editor.project"),
        ("Hierarchy", "editor.hierarchy"),
        ("Inspector", "editor.inspector"),
        ("Scene", "editor.scene"),
        ("Game", "editor.game"),
        ("Assets", "editor.assets"),
        ("Console", "editor.console"),
        ("Prefab Editor", "editor.prefab"),
        ("Asset Browser", "editor.asset_browser"),
    ]
    .into_iter()
    .map(|(label, descriptor_id)| MenuItemModel {
        label: label.to_string(),
        action: MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)),
        binding: menu_action_binding(&MenuAction::OpenView(ViewDescriptorId::new(descriptor_id))),
        shortcut: None,
        enabled: true,
    })
    .collect()
}
