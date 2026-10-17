use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

pub(super) fn insert_unreal_editor_bindings(bindings: &mut BTreeMap<String, EditorUiBinding>) {
    for (view_id, control_id, action_id) in [
        ("UnrealMenu", "File", "workbench.unreal.menu.file.open"),
        ("UnrealMenu", "Edit", "workbench.unreal.menu.edit.open"),
        ("UnrealMenu", "Window", "workbench.unreal.menu.window.open"),
        ("UnrealMenu", "Tools", "workbench.unreal.menu.tools.open"),
        ("UnrealMenu", "Build", "workbench.unreal.menu.build.open"),
        ("UnrealMenu", "Help", "workbench.unreal.menu.help.open"),
        ("UnrealToolbar", "Modes", "workbench.unreal.modes.open"),
        (
            "UnrealViewportToolbarOverflow",
            "Toggle",
            "workbench.unreal.viewport.overflow.open",
        ),
        (
            "Unreal",
            "FindInContentBrowser",
            "workbench.content_drawer.sync_selection",
        ),
        (
            "Unreal",
            "SelectContentAsset",
            "workbench.content_drawer.select_asset",
        ),
        ("UnrealToolbar", "Add", "workbench.unreal.add.open"),
        (
            "UnrealToolbar",
            "Blueprints",
            "workbench.unreal.blueprints.open",
        ),
        (
            "UnrealToolbar",
            "Cinematics",
            "workbench.unreal.cinematics.open",
        ),
        (
            "UnrealToolbar",
            "Settings",
            "workbench.unreal.settings.open",
        ),
        (
            "Unreal",
            "ToggleContentDrawer",
            "workbench.content_drawer.toggle",
        ),
        (
            "Unreal",
            "DismissContentDrawer",
            "workbench.content_drawer.dismiss",
        ),
        (
            "Unreal",
            "FocusContentSearch",
            "workbench.content_drawer.focus_search",
        ),
    ] {
        bindings.insert(
            format!("{view_id}/{control_id}"),
            EditorUiBinding::new(
                view_id,
                control_id,
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action(action_id),
            ),
        );
    }
}
