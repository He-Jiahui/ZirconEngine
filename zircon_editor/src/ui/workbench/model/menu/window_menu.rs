use crate::core::editor_event::{MenuAction, ViewDescriptorId};
use crate::ui::workbench::event::menu_action_binding;

use super::super::menu_item_model::operation_path_for_menu_action;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_window_menu() -> MenuModel {
    let debug_observatory_action =
        MenuAction::OpenView(ViewDescriptorId::new("editor.debug_observatory"));
    let mut items = functional_editor_window_items();
    items.push(MenuItemModel::leaf(
        "Debug Observatory",
        Some(debug_observatory_action.clone()),
        menu_action_binding(&debug_observatory_action),
        operation_path_for_menu_action(&debug_observatory_action),
        None,
        true,
    ));
    items.push(MenuItemModel::leaf(
        "Reset Layout",
        Some(MenuAction::ResetLayout),
        menu_action_binding(&MenuAction::ResetLayout),
        operation_path_for_menu_action(&MenuAction::ResetLayout),
        None,
        true,
    ));

    MenuModel {
        label: "Window".to_string(),
        items,
    }
}

fn functional_editor_window_items() -> Vec<MenuItemModel> {
    [
        ("Prefab Editor", "editor.prefab_editor_window"),
        ("Material Editor", "editor.material_editor_window"),
        ("UI Component Showcase", "editor.ui_component_showcase"),
        ("Material Demo", "editor.material_demo_window"),
        ("UI Asset Editor", "editor.ui_asset_editor_window"),
        ("Animation Editor", "editor.animation_editor_window"),
        ("Asset Browser", "editor.asset_browser_window"),
        ("Diagnostics", "editor.diagnostics_window"),
    ]
    .into_iter()
    .map(|(label, descriptor_id)| {
        let action = MenuAction::OpenView(ViewDescriptorId::new(descriptor_id));
        MenuItemModel::leaf(
            label,
            Some(action.clone()),
            menu_action_binding(&action),
            operation_path_for_menu_action(&action),
            None,
            true,
        )
    })
    .collect()
}
