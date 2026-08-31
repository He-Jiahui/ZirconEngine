const WORKBENCH_TOP_TOOLBAR_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui"
));
const WORKBENCH_WINDOW_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/windows/workbench_window.zui"
));
const WORKBENCH_STRICT_THEME: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/theme/editor_workbench_strict.zui"
));

#[test]
fn workbench_run_controls_keep_their_tokenized_mvp_command_contract() {
    for required in [
        "res://ui/editor/theme/editor_tokens.zui",
        "WorkbenchToolbarRunGroup",
        "WorkbenchRunPlay",
        "WorkbenchRunMode",
        "$editor.density.gap.small",
        "$editor.control.height.compact",
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
fn workbench_popup_menus_use_shared_density_tokens_and_popup_surface() {
    for required in [
        "$editor.density.gap.medium",
        "$editor.density.gap.small",
        "$editor.control.height.compact",
        "control_id = \"WorkbenchRunModeMenu\"",
        "Play In Editor|icon=play",
        "Simulate|icon=play",
    ] {
        assert!(
            WORKBENCH_WINDOW_TEMPLATE.contains(required),
            "Workbench popup presentation must consume `{required}`"
        );
    }

    for authored_business_state in [
        "Play In Editor|checked,icon=play",
        "Default Layout|checked,disabled,icon=grid",
    ] {
        assert!(
            !WORKBENCH_WINDOW_TEMPLATE.contains(authored_business_state),
            "Workbench popup templates must not author live state `{authored_business_state}`"
        );
    }

    for required in [
        "background_color = \"$editor.popup\"",
        "border_color = \"$workbench_border\"",
        "radius = \"$editor.control.radius.panel\"",
    ] {
        assert!(
            WORKBENCH_STRICT_THEME.contains(required),
            "Workbench popup theme must consume `{required}`"
        );
    }

    for legacy_primitive in [
        "layout_padding_left = 10.0",
        "layout_padding_right = 10.0",
        "layout_spacing = 6.0",
        "layout_min_height = 30.0",
    ] {
        assert!(
            !WORKBENCH_WINDOW_TEMPLATE.contains(legacy_primitive),
            "Workbench window must not retain popup-local `{legacy_primitive}`"
        );
    }
}
