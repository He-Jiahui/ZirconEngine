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
    include_str!("../../../../../assets/ui/editor/unreal/editor_shell.zui");
const WORKBENCH_STRICT_THEME: &str =
    include_str!("../../../../../assets/ui/theme/editor_workbench_strict.zui");
const EDITOR_TOKENS: &str = include_str!("../../../../../assets/ui/editor/theme/editor_tokens.zui");
const UNREAL_TOOLBAR: &str =
    include_str!("../../../../../assets/ui/editor/unreal/components/toolbar.zui");
const UNREAL_STATUS_BAR: &str =
    include_str!("../../../../../assets/ui/editor/unreal/components/status_bar.zui");
const UNREAL_CONTENT_DRAWER: &str =
    include_str!("../../../../../assets/ui/editor/unreal/components/content_drawer.zui");

#[test]
fn unreal_shell_root_uses_semantic_host_mount_and_starship() {
    assert!(ROOT_WORKBENCH_SHELL.contains("component = \"UiHostWindow\""));
    assert!(ROOT_WORKBENCH_SHELL.contains("control_id = \"UiHostWindowRoot\""));
    assert!(ROOT_WORKBENCH_SHELL.contains("component = \"DocumentHost\""));
    assert!(ROOT_WORKBENCH_SHELL.contains("component = \"PaneSurface\""));
    assert!(!ROOT_WORKBENCH_SHELL.contains("activity_rail"));
    assert!(!ROOT_WORKBENCH_SHELL.contains("component_drawer"));
    assert!(WORKBENCH_STRICT_THEME.contains("res://ui/editor/theme/editor_tokens.zui"));
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
        ".workbench-strip",
        ".workbench-panel",
        ".workbench-module-canvas",
        ".workbench-left-panel",
        ".workbench-right-panel",
        ".workbench-section-title",
        ".workbench-property-section",
        ".workbench-component-drawer",
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
fn unreal_toolbar_uses_one_bottom_separator_and_fixed_48px_band() {
    assert!(UNREAL_TOOLBAR.contains("separator_edge = \"bottom\""));
    assert!(UNREAL_TOOLBAR.contains("height = { min = 48.0, preferred = 48.0, max = 48.0"));
    assert!(!UNREAL_TOOLBAR.contains("workbench_top_toolbar.zui"));
}

#[test]
fn unreal_status_bar_uses_one_top_separator_and_retains_status_identity() {
    assert!(UNREAL_STATUS_BAR.contains("control_id = \"UnrealEditorStatusBar\""));
    assert!(UNREAL_STATUS_BAR.contains("separator_edge = \"top\""));
    assert!(UNREAL_STATUS_BAR.contains("control_id = \"WorkbenchStatusReady\""));
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
            "0.0",
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
            "$editor.control.radius.control",
            "$editor.control.border_width",
        ),
        (
            ".workbench-popup-menu",
            "$editor.popup",
            "$editor.control.radius.panel",
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
    assert!(
        panel_header.contains("corner_radius = 0.0"),
        "panel header must remain a continuous square pane band rather than a nested card"
    );
    assert!(
        panel_header.contains("border_width = 0.0")
            && panel_header.contains("separator_edge = \"bottom\"")
            && panel_header.contains("separator_thickness = \"$editor.control.border_width\""),
        "panel header must use one bottom separator instead of a four-sided inner frame"
    );

    let section_title = workbench_asset!("primitives/chrome/workbench_section_title.zui");
    assert!(
        strict_theme_rule(".workbench-section-title")
            .contains("radius = \"$editor.control.radius.small\""),
        "section titles must use the shared small-radius token"
    );
    assert!(
        !section_title.contains("corner_radius = 0.0"),
        "section title component must not reintroduce a square-corner override"
    );
}

#[test]
fn selection_open_and_press_recipes_do_not_borrow_the_focus_outline() {
    for selector in [
        ".workbench-icon-button:selected",
        ".workbench-rail-button:selected",
        ".workbench-tab:checked",
        ".workbench-tab:selected",
        ".workbench-tree-item:selected",
        ".workbench-list-row:selected",
        ".workbench-table-row:selected",
        ".workbench-segmented-control:selected",
        ".workbench-control-button:selected",
    ] {
        let rule = strict_theme_rule(selector);
        assert!(
            rule.contains("background_color = \"$workbench_selected\""),
            "{selector} must express persistent selection with the neutral selection surface"
        );
        assert!(
            rule.contains("border_color = \"$workbench_border\""),
            "{selector} must retain the low-emphasis control border"
        );
        assert!(
            !rule.contains("border_color = \"$workbench_accent\"")
                && !rule.contains("border_color = \"$editor.focus.ring\""),
            "{selector} must not render a full accent/focus outline"
        );
    }

    for selector in [
        ".workbench-icon-button:pressed",
        ".workbench-toolbar-button:pressed",
        ".workbench-rail-button:pressed",
        ".workbench-tab:pressed",
        ".workbench-tree-item:pressed",
        ".workbench-list-row:pressed",
        ".workbench-table-row:pressed",
        ".workbench-segmented-control:pressed",
        ".workbench-control-button:pressed",
        ".workbench-dropdown:popup_open",
    ] {
        let rule = strict_theme_rule(selector);
        assert!(
            rule.contains("border_color = \"$workbench_border\""),
            "{selector} must use fill/foreground feedback without borrowing the focus outline"
        );
        assert!(
            !rule.contains("border_color = \"$workbench_accent\"")
                && !rule.contains("border_color = \"$editor.focus.ring\""),
            "{selector} must reserve full accent outlines for focus-visible and validation"
        );
    }
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
    assert!(
        !WORKBENCH_STRICT_THEME.contains(".workbench-focused"),
        "strict theme must not keep a private class that bypasses retained focus-visible state"
    );

    let drawer: toml::Value =
        toml::from_str(UNREAL_CONTENT_DRAWER).expect("the current content drawer must parse");
    let search = &drawer["nodes"]["search"];
    assert_eq!(
        search["props"]["input_focusable"].as_bool(),
        Some(true),
        "the real drawer search must participate in retained keyboard focus"
    );
    assert!(
        search["props"].get("focus_visible").is_none() && search["props"].get("focused").is_none(),
        "the authored search must let retained focus state own the focus ring"
    );
    assert!(
        search["events"].as_array().is_some_and(|events| {
            events.iter().any(|event| {
                event["event"].as_str() == Some("Change")
                    && event["route"].as_str() == Some("workbench.asset.set_search_query")
            })
        }),
        "focusable drawer search must keep its actual asset query binding"
    );
}

#[test]
fn unreal_content_drawer_uses_focus_loss_dismissal_and_search_identity() {
    assert!(UNREAL_CONTENT_DRAWER.contains("control_id = \"SearchEdited\""));
    assert!(UNREAL_CONTENT_DRAWER.contains("control_id = \"ContentDrawerClose\""));
    assert!(UNREAL_CONTENT_DRAWER.contains("route = \"workbench.content_drawer.dismiss\""));
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
