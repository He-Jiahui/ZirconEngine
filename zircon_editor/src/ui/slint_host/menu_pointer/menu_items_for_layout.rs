use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::menu_item_spec::MenuItemSpec;

pub(in crate::ui::slint_host::menu_pointer) fn menu_items_for_layout(
    layout: &HostMenuPointerLayout,
    menu_index: usize,
) -> Vec<MenuItemSpec> {
    if let Some(items) = layout.menus.get(menu_index) {
        return items.clone();
    }

    match menu_index {
        0 => vec![
            menu_action("OpenProject", true),
            menu_action("SaveProject", layout.save_project_enabled),
            menu_action("SaveLayout", true),
            menu_action("ResetLayout", true),
            disabled_item(),
        ],
        1 => vec![
            menu_action("Undo", layout.undo_enabled),
            menu_action("Redo", layout.redo_enabled),
        ],
        2 => vec![
            menu_action("CreateNode.Cube", true),
            menu_action("CreateNode.Camera", true),
            menu_action("CreateNode.DirectionalLight", true),
            menu_action("DeleteSelected", layout.delete_enabled),
        ],
        3 => vec![
            menu_action("EnterPlayMode", true),
            menu_action("ExitPlayMode", false),
        ],
        4 => vec![
            menu_action("OpenView.editor.project", true),
            menu_action("OpenView.editor.hierarchy", true),
            menu_action("OpenView.editor.inspector", true),
            menu_action("OpenView.editor.scene", true),
            menu_action("OpenView.editor.game", true),
            menu_action("OpenView.editor.assets", true),
            menu_action("OpenView.editor.console", true),
            menu_action("OpenView.editor.prefab", true),
        ],
        5 => {
            let mut items = vec![
                menu_action(format!("SavePreset.{}", layout.resolved_preset_name), true),
                menu_action("ResetLayout", true),
            ];
            items.extend(
                layout
                    .preset_names
                    .iter()
                    .map(|preset| menu_action(format!("LoadPreset.{preset}"), true)),
            );
            items
        }
        6 => vec![menu_action("OpenView.editor.asset_browser", true)],
        _ => Vec::new(),
    }
}

fn menu_action(action_id: impl Into<String>, enabled: bool) -> MenuItemSpec {
    MenuItemSpec {
        action_id: enabled.then(|| action_id.into()),
        enabled,
        children: Vec::new(),
    }
}

fn disabled_item() -> MenuItemSpec {
    MenuItemSpec {
        action_id: None,
        enabled: false,
        children: Vec::new(),
    }
}
