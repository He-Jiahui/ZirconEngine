//! Verify the retained Unreal toolbar owns the run controls after the Workbench cutover.

const WORKBENCH_TOP_TOOLBAR_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/unreal/components/toolbar.zui"
));
const WORKBENCH_WINDOW_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/unreal/level_editor_window.zui"
));
const WORKBENCH_STRICT_THEME: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/theme/editor_workbench_strict.zui"
));

#[test]
fn workbench_run_controls_keep_their_tokenized_mvp_command_contract() {
    for required in [
        "res://ui/editor/unreal/starship.zui",
        "WorkbenchToolbarCommandRow",
        "WorkbenchRunPlay",
        "WorkbenchRunMode",
        "WorkbenchRunStop",
        "height = { min = 48.0, preferred = 48.0, max = 48.0",
        "gap = 4.0",
        "action = { action = \"runtime.play_mode.enter\" }",
        "route = \"workbench.run.open_mode_menu\"",
    ] {
        assert!(
            WORKBENCH_TOP_TOOLBAR_TEMPLATE.contains(required),
            "Workbench run controls must preserve `{required}`"
        );
    }
}

#[test]
fn unreal_level_editor_mounts_native_popup_and_shell_regions() {
    for required in [
        "res://ui/editor/unreal/components/toolbar.zui#UnrealLevelEditorToolbar",
        "res://ui/editor/unreal/components/menus.zui#UnrealLevelEditorMenus",
        "res://ui/editor/unreal/components/menu_popups.zui#UnrealLevelEditorMenuPopups",
        "res://ui/editor/unreal/starship.zui",
        "control_id = \"WorkbenchWindowRoot\"",
        "{ node = \"content_drawer\" }",
        "{ node = \"menu_popups\" }",
        "control_id = \"UnrealEditorMenuPopups\"",
        "z_index = 180",
    ] {
        assert!(
            WORKBENCH_WINDOW_TEMPLATE.contains(required),
            "Unreal level-editor window must retain `{required}`"
        );
    }

    assert!(
        !WORKBENCH_WINDOW_TEMPLATE.contains("workbench_window.zui")
            && !WORKBENCH_WINDOW_TEMPLATE.contains("workbench_shell.zui"),
        "Unreal level-editor window must not refer back to retired Workbench roots"
    );
    assert!(
        WORKBENCH_STRICT_THEME.contains("res://ui/editor/theme/editor_tokens.zui"),
        "shared editor theme remains available to retained compatibility surfaces"
    );
}
