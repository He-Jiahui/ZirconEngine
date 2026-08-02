use super::support::assert_tokenized_assets;

macro_rules! workbench_asset {
    ($path:literal) => {
        include_str!(concat!(
            "../../../../../assets/ui/editor/components/workbench/",
            $path
        ))
    };
}

const ROOT_WORKBENCH_SHELL: &str =
    include_str!("../../../../../assets/ui/editor/host/workbench_shell.zui");
const WORKBENCH_STRICT_THEME: &str =
    include_str!("../../../../../assets/ui/theme/editor_workbench_strict.zui");

#[test]
fn root_workbench_chrome_uses_shared_metrics_and_control_tokens() {
    assert_tokenized_assets(&[(
        "workbench_shell.zui",
        ROOT_WORKBENCH_SHELL,
        &[
            "$editor.control.height.default",
            "$editor.control.height.dense",
            "$editor.control.border_width",
            "$editor.control.radius.control",
            "$editor.density.toolbar_action_width",
            "$editor.density.toolbar_wide_action_width",
            "$editor.density.row_height",
        ],
    )]);

    for legacy_extent in [
        "min = 24.0",
        "preferred = 24.0",
        "max = 24.0",
        "min = 25.0",
        "preferred = 25.0",
        "max = 25.0",
        "min = 76.0",
        "preferred = 76.0",
        "max = 76.0",
        "min = 96.0",
        "preferred = 96.0",
        "max = 96.0",
    ] {
        assert!(
            !ROOT_WORKBENCH_SHELL.contains(legacy_extent),
            "root workbench chrome must not retain {legacy_extent}"
        );
    }

    assert!(WORKBENCH_STRICT_THEME.contains("res://ui/editor/theme/editor_tokens.zui"));
    assert!(
        WORKBENCH_STRICT_THEME
            .matches("radius = \"$editor.control.radius.control\"")
            .count()
            >= 11,
        "button state variants must share the control-radius token"
    );
    assert!(
        WORKBENCH_STRICT_THEME
            .matches("radius = \"$editor.control.radius.small\"")
            .count()
            >= 3,
        "field state variants must share the small-radius token"
    );
    assert!(
        WORKBENCH_STRICT_THEME.contains("radius = \"$editor.control.radius.pill\""),
        "toggle controls must use the shared pill-radius token"
    );
    for legacy_radius in ["radius = 5.0", "radius = 999.0"] {
        assert!(
            !WORKBENCH_STRICT_THEME.contains(legacy_radius),
            "strict workbench controls must not retain {legacy_radius}"
        );
    }
}

#[test]
fn interactive_workbench_states_share_the_control_border_token() {
    for selector in [
        ".workbench-hover",
        ".workbench-tab:hovered",
        ".workbench-slider:hovered",
        ".workbench-tree-item:hovered",
        ".workbench-list-row:hovered",
        ".workbench-table-row:hovered",
        ".workbench-check:hovered",
        ".workbench-radio:hovered",
        ".workbench-toggle:hovered",
        ".workbench-segmented-control:hovered",
        ".workbench-control-button:hovered",
        ".workbench-tab:pressed",
        ".workbench-tree-item:pressed",
        ".workbench-list-row:pressed",
        ".workbench-table-row:pressed",
        ".workbench-slider:pressed",
        ".workbench-check:pressed",
        ".workbench-radio:pressed",
        ".workbench-toggle:pressed",
        ".workbench-segmented-control:pressed",
        ".workbench-control-button:pressed",
        ".workbench-tabs",
        ".workbench-tab-active",
        ".workbench-tab:checked",
        ".workbench-tab:selected",
        ".workbench-tree-item:selected",
        ".workbench-list-row:selected",
        ".workbench-table-row:selected",
        ".workbench-check:checked",
        ".workbench-radio:checked",
        ".workbench-toggle:checked",
        ".workbench-segmented-control:selected",
        ".workbench-control-button:selected",
        ".workbench-axis-value-field",
        ".workbench-topbar",
        ".workbench-strip",
        ".workbench-rail",
        ".workbench-panel",
        ".workbench-module-canvas",
        ".workbench-left-panel",
        ".workbench-right-panel",
        ".workbench-section-title",
        ".workbench-property-section",
        ".workbench-component-drawer",
        ".workbench-status",
        ".workbench-status-right-control",
        ".workbench-status-right-icon",
        ".workbench-overlay-region",
    ] {
        let rule = strict_theme_rule(selector);
        assert!(
            rule.contains("border_width = \"$editor.control.border_width\""),
            "{selector} must use the shared control border width"
        );
        assert!(
            !rule.contains("border_width = 1.0"),
            "{selector} must not retain a local border width"
        );
    }
}

#[test]
fn strict_theme_uses_atomic_radius_tokens_for_compact_control_details() {
    for (selector, radius_token) in [
        (
            ".workbench-axis-value-field",
            "$editor.control.radius.control",
        ),
        (".workbench-module-canvas", "$editor.control.radius.control"),
        (".workbench-progress-bar", "$editor.control.radius.small"),
    ] {
        assert!(
            strict_theme_rule(selector).contains(&format!("radius = \"{radius_token}\"")),
            "{selector} must use {radius_token}"
        );
    }
}

fn strict_theme_rule(selector: &str) -> &str {
    let selector_line = format!("selector = \"{selector}\"");
    let (_, rule) = WORKBENCH_STRICT_THEME
        .split_once(&selector_line)
        .unwrap_or_else(|| panic!("strict theme is missing {selector}"));
    rule.split("[[stylesheets.rules]]")
        .next()
        .unwrap_or_default()
}
