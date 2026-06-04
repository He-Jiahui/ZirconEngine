use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::menu_item_spec::MenuItemSpec;

pub(in crate::ui::retained_host::menu_pointer) fn menu_items_for_layout(
    layout: &HostMenuPointerLayout,
    menu_index: usize,
) -> Vec<MenuItemSpec> {
    if let Some(items) = layout.menus.get(menu_index) {
        return items.clone();
    }

    match menu_index {
        0 => vec![
            menu_action("workbench.project.open", true),
            menu_action("workbench.project.save", layout.save_project_enabled),
            menu_action("workbench.layout.save", true),
            menu_action("workbench.layout.reset", true),
            disabled_item(),
        ],
        1 => vec![
            menu_action("workbench.history.undo", layout.undo_enabled),
            menu_action("workbench.history.redo", layout.redo_enabled),
        ],
        2 => vec![
            menu_action("workbench.scene.node.create.cube", true),
            menu_action("workbench.scene.node.create.camera", true),
            menu_action("workbench.scene.node.create.ambient_light", true),
            menu_action("workbench.scene.node.create.directional_light", true),
            menu_action("workbench.scene.node.create.point_light", true),
            menu_action("workbench.scene.node.create.rect_light", true),
            menu_action("workbench.scene.node.create.spot_light", true),
            menu_action("workbench.selection.delete_selected", layout.delete_enabled),
        ],
        3 => vec![
            menu_action("workbench.play_mode.enter", true),
            menu_action("workbench.play_mode.exit", false),
        ],
        4 => vec![
            menu_action("workbench.view.open.editor.project", true),
            menu_action("workbench.view.open.editor.hierarchy", true),
            menu_action("workbench.view.open.editor.inspector", true),
            menu_action("workbench.view.open.editor.scene", true),
            menu_action("workbench.view.open.editor.game", true),
            menu_action("workbench.view.open.editor.assets", true),
            menu_action("workbench.view.open.editor.console", true),
            menu_action("workbench.view.open.editor.prefab", true),
        ],
        5 => {
            let mut items = vec![
                menu_action(
                    format!(
                        "workbench.layout.preset.save.{}",
                        layout.resolved_preset_name
                    ),
                    true,
                ),
                menu_action("workbench.layout.reset", true),
            ];
            items.extend(
                layout.preset_names.iter().map(|preset| {
                    menu_action(format!("workbench.layout.preset.load.{preset}"), true)
                }),
            );
            items.push(menu_action(
                "workbench.view.open.editor.debug_observatory",
                true,
            ));
            items
        }
        6 => vec![menu_action(
            "workbench.view.open.editor.asset_browser",
            true,
        )],
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
