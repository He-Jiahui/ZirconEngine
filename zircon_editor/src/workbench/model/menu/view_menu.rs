use crate::view::ViewDescriptorId;
use crate::workbench::event::menu_action_binding;

use super::super::menu_action::MenuAction;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_view_menu() -> MenuModel {
    MenuModel {
        label: "View".to_string(),
        items: builtin_view_menu_items(),
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
