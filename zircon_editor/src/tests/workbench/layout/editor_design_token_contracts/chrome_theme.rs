use super::support::assert_tokenized_assets;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

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
const EDITOR_TOKENS: &str = include_str!("../../../../../assets/ui/editor/theme/editor_tokens.zui");

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
            "$editor.chrome.activity_rail.width",
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
fn editor_density_asset_and_runtime_registry_have_identical_named_values() {
    let document: toml::Value = toml::from_str(EDITOR_TOKENS).expect("editor tokens must parse");
    let density = document
        .get("density")
        .and_then(toml::Value::as_table)
        .expect("editor tokens must declare [density]");
    let names = document
        .get("names")
        .and_then(|value| value.get("density"))
        .and_then(toml::Value::as_table)
        .expect("editor tokens must declare [names.density]");
    assert_eq!(
        density.len(),
        names.len(),
        "every editor density value must have one canonical token name"
    );

    let runtime_values = EditorDesignTokens::workbench_dark().cascade_token_values();
    for (field_name, token_name) in names {
        let token_name = token_name
            .as_str()
            .unwrap_or_else(|| panic!("density token name `{field_name}` must be a string"));
        let asset_value = density
            .get(field_name)
            .and_then(toml::Value::as_float)
            .unwrap_or_else(|| panic!("density value `{field_name}` must be a float"));
        let runtime_value = runtime_values
            .get(token_name)
            .and_then(toml::Value::as_float)
            .unwrap_or_else(|| panic!("Runtime must register `{token_name}`"));
        assert!(
            (asset_value - runtime_value).abs() < 0.0001,
            "density token `{token_name}` drifted: asset={asset_value}, Runtime={runtime_value}"
        );
    }
}

#[test]
fn strict_theme_raw_colors_are_confined_to_viewport_and_axis_visuals() {
    let mut raw_color_count = 0;
    for (line_index, line) in WORKBENCH_STRICT_THEME.lines().enumerate() {
        if !line.contains("= \"#") {
            continue;
        }
        raw_color_count += 1;
        let token_name = line
            .split_once('=')
            .map(|(name, _)| name.trim())
            .unwrap_or_default();
        assert!(
            token_name.starts_with("workbench_viewport_")
                || token_name.starts_with("workbench_axis_"),
            "strict theme line {} keeps a semantic raw color in `{token_name}`",
            line_index + 1
        );
    }
    assert!(
        raw_color_count > 0,
        "viewport and axis visuals must exercise the raw-color allowlist"
    );

    for semantic_assignment in [
        "workbench_toast_surface = \"$editor.popup\"",
        "workbench_status_no_errors_fill = \"$editor.semantic.success\"",
        "workbench_field_component_focus_border = \"$editor.focus.ring\"",
        "workbench_field_component_disabled = \"$editor.surface.disabled\"",
        "workbench_toggle_checked_track = \"$editor.surface.selected\"",
        "workbench_warning_surface = \"$editor.semantic.warning.container\"",
        "workbench_error_surface = \"$editor.semantic.error.container\"",
    ] {
        assert!(
            WORKBENCH_STRICT_THEME.contains(semantic_assignment),
            "strict theme must retain semantic assignment `{semantic_assignment}`"
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

#[test]
fn workbench_family_recipes_share_semantic_surface_and_shape_roles() {
    for (selector, surface, radius, border_width) in [
        (
            ".workbench-toolbar-button",
            "$workbench_panel",
            "$editor.control.radius.control",
            "$editor.control.border_width",
        ),
        (
            ".workbench-panel-header",
            "$workbench_panel_raised",
            "0.0",
            "$editor.control.border_width",
        ),
        (
            ".workbench-list-row",
            "$workbench_panel",
            "$editor.control.radius.control",
            "0.0",
        ),
        (
            ".workbench-field",
            "$workbench_field",
            "$editor.control.radius.small",
            "$editor.control.border_width",
        ),
        (
            ".workbench-popup-menu",
            "$editor.popup",
            "$editor.control.radius.control",
            "$editor.control.border_width",
        ),
    ] {
        let rule = strict_theme_rule(selector);
        assert!(
            rule.contains(&format!("background_color = \"{surface}\"")),
            "{selector} must use the canonical family surface {surface}"
        );
        assert!(
            rule.contains(&format!("radius = {radius}"))
                || rule.contains(&format!("radius = \"{radius}\"")),
            "{selector} must use the canonical family radius {radius}"
        );
        assert!(
            rule.contains(&format!("border_width = {border_width}"))
                || rule.contains(&format!("border_width = \"{border_width}\"")),
            "{selector} must use the canonical family border width {border_width}"
        );
    }

    let panel_header = workbench_asset!("composites/chrome/workbench_panel_header.zui");
    assert_eq!(
        panel_header
            .matches("$editor.chrome.panel_header.height")
            .count(),
        3,
        "panel header min/preferred/max must share one stable chrome metric"
    );
    assert!(
        !panel_header.contains("$editor.control.height.dense")
            && !panel_header.contains("$editor.control.height.compact"),
        "panel header height must not drift through generic control metrics"
    );
}

#[test]
fn focus_visible_rules_are_border_only_overlays_after_primary_state_rules() {
    let last_primary_rule = WORKBENCH_STRICT_THEME
        .find("selector = \".workbench-dropdown:popup_open\"")
        .expect("strict theme must retain the dropdown open recipe");

    for selector in [
        ".workbench-component-property-row:focus-visible",
        ".workbench-field:focus-visible",
        ".workbench-component-field:focus-visible",
        ".workbench-dropdown:focus-visible",
    ] {
        let selector_line = format!("selector = \"{selector}\"");
        let selector_offset = WORKBENCH_STRICT_THEME
            .find(&selector_line)
            .unwrap_or_else(|| panic!("strict theme is missing {selector}"));
        assert!(
            selector_offset > last_primary_rule,
            "{selector} must cascade after selected/open primary-state recipes"
        );

        let rule = strict_theme_rule(selector);
        assert!(
            rule.contains("border_width = \"$editor.control.border_width\""),
            "{selector} must use the shared outline width"
        );
        assert!(
            rule.contains("border_color = \"$editor.focus.ring\"")
                || rule.contains("border_color = \"$workbench_field_component_focus_border\"",),
            "{selector} must use the semantic focus-ring role"
        );
        assert!(
            !rule.contains("background_color") && !rule.contains("foreground_color"),
            "{selector} must not replace primary fill or foreground identity"
        );
    }

    assert!(
        !WORKBENCH_STRICT_THEME.contains(":focused\""),
        "strict theme must not style pointer/programmatic focus as keyboard-visible focus"
    );
}

pub(super) fn strict_theme_rule(selector: &str) -> &str {
    let selector_line = format!("selector = \"{selector}\"");
    let (_, rule) = WORKBENCH_STRICT_THEME
        .split_once(&selector_line)
        .unwrap_or_else(|| panic!("strict theme is missing {selector}"));
    rule.split("[[stylesheets.rules]]")
        .next()
        .unwrap_or_default()
}
