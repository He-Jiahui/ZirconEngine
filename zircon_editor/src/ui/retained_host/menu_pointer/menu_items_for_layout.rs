use std::borrow::Cow;

use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::menu_item_spec::MenuItemSpec;

#[cfg(test)]
#[path = "menu_items_for_layout/preset_capacity_tests.rs"]
mod preset_capacity_tests;

pub(in crate::ui::retained_host::menu_pointer) fn menu_items_for_layout<'a>(
    layout: &'a HostMenuPointerLayout,
    menu_index: usize,
) -> Cow<'a, [MenuItemSpec]> {
    if let Some(items) = layout.menus.get(menu_index) {
        return Cow::Borrowed(items);
    }

    Cow::Owned(match menu_index {
        0 => vec![
            menu_action("file.project.open", true),
            menu_action("file.project.save", layout.save_project_enabled),
            menu_action("window.layout.save", true),
            menu_action("window.layout.reset", true),
            disabled_item(),
        ],
        1 => vec![
            menu_action("edit.history.undo", layout.undo_enabled),
            menu_action("edit.history.redo", layout.redo_enabled),
        ],
        2 => vec![
            menu_action("scene.node.create_cube", true),
            menu_action("scene.node.create_camera", true),
            menu_action("scene.node.create_ambient_light", true),
            menu_action("scene.node.create_directional_light", true),
            menu_action("scene.node.create_point_light", true),
            menu_action("scene.node.create_rect_light", true),
            menu_action("scene.node.create_spot_light", true),
            menu_action("scene.node.delete_selected", layout.delete_enabled),
        ],
        3 => vec![
            menu_action("runtime.play_mode.enter", true),
            menu_action("runtime.play_mode.exit", false),
        ],
        4 => vec![
            menu_action("view.project.open", true),
            menu_action("view.hierarchy.open", true),
            menu_action("view.inspector.open", true),
            menu_action("view.scene.open", true),
            menu_action("view.game.open", true),
            menu_action("view.assets.open", true),
            menu_action("view.console.open", true),
            menu_action("view.prefab.open", true),
        ],
        5 => {
            let mut items =
                Vec::with_capacity(menu_preset_item_capacity(layout.preset_names.len()));
            items.push(menu_action(
                format!(
                    "workbench.layout.preset.save.{}",
                    layout.resolved_preset_name
                ),
                true,
            ));
            items.push(menu_action("window.layout.reset", true));
            items.extend(
                layout.preset_names.iter().map(|preset| {
                    menu_action(format!("workbench.layout.preset.load.{preset}"), true)
                }),
            );
            items.push(menu_action("window.debug_observatory.open", true));
            items
        }
        6 => vec![menu_action("view.asset_browser.open", true)],
        _ => Vec::new(),
    })
}

fn menu_preset_item_capacity(preset_count: usize) -> usize {
    preset_count.saturating_add(3)
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
