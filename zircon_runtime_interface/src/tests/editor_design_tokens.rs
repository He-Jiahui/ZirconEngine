use crate::ui::design_tokens::{
    EditorDensityTokens, EditorDesignTokens, EditorStateColorRole, EditorTypographyTokens,
    EditorUtilityTabTextRole, EDITOR_WORKBENCH_TOKENS_ID,
};
use crate::ui::style::{UiPainterFamily, UiPainterResolvedState, UiPainterState, UiRgbaColor};
use toml::Value;

#[test]
fn editor_design_tokens_encode_workbench_style_notes_palette_and_density() {
    let tokens = EditorDesignTokens::workbench_dark();

    assert_eq!(tokens.id, EDITOR_WORKBENCH_TOKENS_ID);
    assert_eq!(
        tokens.palette.surface,
        [
            UiRgbaColor::from_u8(17, 20, 22, 255),
            UiRgbaColor::from_u8(23, 26, 29, 255),
            UiRgbaColor::from_u8(27, 31, 35, 255),
            UiRgbaColor::from_u8(37, 43, 49, 255),
        ]
    );
    assert_eq!(
        tokens.palette.accent,
        UiRgbaColor::from_u8(60, 199, 214, 255)
    );
    assert_eq!(
        tokens.palette.surface_recessed,
        UiRgbaColor::from_u8(15, 19, 22, 255)
    );
    assert_eq!(
        tokens.palette.surface_hover,
        UiRgbaColor::from_u8(42, 48, 54, 255)
    );
    assert_eq!(
        tokens.palette.surface_selected,
        UiRgbaColor::from_u8(23, 57, 66, 255)
    );
    assert_eq!(
        tokens.palette.separator_strong,
        UiRgbaColor::from_u8(65, 75, 84, 255)
    );
    assert_eq!(tokens.palette.popup, UiRgbaColor::from_u8(20, 22, 24, 255));
    assert_eq!(tokens.controls.border_width, 1.0);
    assert_eq!(tokens.controls.large_height, 48.0);
    assert_eq!(tokens.controls.compact_height, 30.0);
    assert_eq!(tokens.controls.dense_height, 28.0);
    assert_eq!(tokens.typography.ui_family, "system-ui");
    assert_eq!(tokens.typography.code_family, "monospace");
    assert_eq!(
        tokens.typography.utility_tab_text_role,
        EditorUtilityTabTextRole::Ui
    );
    let slate_points_to_logical_pixels = 96.0 / 72.0;
    assert!((tokens.typography.body_size - 10.0 * slate_points_to_logical_pixels).abs() < 0.001);
    assert!((tokens.typography.caption_size - 8.0 * slate_points_to_logical_pixels).abs() < 0.001);
    assert!((tokens.typography.overlay_size - 9.0 * slate_points_to_logical_pixels).abs() < 0.001);
    assert!((tokens.typography.title_size - 14.0 * slate_points_to_logical_pixels).abs() < 0.001);
    assert_eq!(tokens.typography.medium_weight, 500);
    assert_eq!(tokens.typography.emphasis_weight, 700);
    assert_eq!(
        tokens.typography.line_height,
        EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO
    );
    assert_eq!(tokens.density.gap_xsmall, 2.0);
    assert_eq!(tokens.density.gap_small, 4.0);
    assert_eq!(tokens.density.drawer_padding, 12.0);
    assert_eq!(tokens.density.activity_rail_width, 72.0);
    assert_eq!(
        tokens.density.row_height,
        EditorDensityTokens::WORKBENCH_ROW_HEIGHT
    );
}

#[test]
fn editor_design_tokens_resolve_named_density_constraint_tokens() {
    let tokens = EditorDesignTokens::workbench_dark();

    assert_eq!(
        tokens.density_value_for_token_name("--left-drawer-width"),
        Some(tokens.density.left_drawer_width)
    );
    assert_eq!(
        tokens.density_value_for_token_name("--right-drawer-width"),
        Some(tokens.density.right_drawer_width)
    );
    assert_eq!(
        tokens.density_value_for_token_name("--bottom-output-height"),
        Some(tokens.density.bottom_output_height)
    );
    assert_eq!(
        tokens.density_value_for_token_name("--breakpoint-narrow-width"),
        Some(tokens.density.breakpoint_narrow_width)
    );
    assert_eq!(
        tokens.density_value_for_token_name("--compact-side-width"),
        Some(tokens.density.compact_side_width)
    );
    assert_eq!(
        tokens.density_value_for_token_name("--minimum-document-width-fraction"),
        Some(tokens.density.minimum_document_width_fraction)
    );
    assert_eq!(
        tokens.density_value_for_token_name("--compact-bottom-max-height"),
        Some(tokens.density.compact_bottom_max_height)
    );
    assert_eq!(
        tokens.density_value_for_token_name("--minimum-window-width"),
        Some(tokens.density.minimum_window_width)
    );
    assert_eq!(tokens.density_value_for_token_name("--unknown"), None);
}

#[test]
fn editor_design_tokens_keep_legacy_density_aliases_equal_to_canonical_names() {
    let tokens = EditorDesignTokens::workbench_dark();
    let expected = [
        (
            "editor.density.left_drawer_width",
            "--left-drawer-width",
            tokens.density.left_drawer_width,
        ),
        (
            "editor.density.right_drawer_width",
            "--right-drawer-width",
            tokens.density.right_drawer_width,
        ),
        (
            "editor.density.bottom_output_height",
            "--bottom-output-height",
            tokens.density.bottom_output_height,
        ),
        (
            "editor.density.breakpoint_ultra_width",
            "--breakpoint-ultra-width",
            tokens.density.breakpoint_ultra_width,
        ),
        (
            "editor.density.breakpoint_narrow_width",
            "--breakpoint-narrow-width",
            tokens.density.breakpoint_narrow_width,
        ),
        (
            "editor.density.breakpoint_wide_width",
            "--breakpoint-wide-width",
            tokens.density.breakpoint_wide_width,
        ),
        (
            "editor.density.compact_side_width",
            "--compact-side-width",
            tokens.density.compact_side_width,
        ),
        (
            "editor.density.ultra_compact_side_width",
            "--ultra-compact-side-width",
            tokens.density.ultra_compact_side_width,
        ),
        (
            "editor.density.compact_left_drawer_max_width",
            "--compact-left-drawer-max-width",
            tokens.density.compact_left_drawer_max_width,
        ),
        (
            "editor.density.compact_right_drawer_max_width",
            "--compact-right-drawer-max-width",
            tokens.density.compact_right_drawer_max_width,
        ),
        (
            "editor.density.compact_side_min_width",
            "--compact-side-min-width",
            tokens.density.compact_side_min_width,
        ),
        (
            "editor.density.minimum_document_width_fraction",
            "--minimum-document-width-fraction",
            tokens.density.minimum_document_width_fraction,
        ),
        (
            "editor.density.ultra_compact_left_drawer_max_width",
            "--ultra-compact-left-drawer-max-width",
            tokens.density.ultra_compact_left_drawer_max_width,
        ),
        (
            "editor.density.ultra_compact_right_drawer_max_width",
            "--ultra-compact-right-drawer-max-width",
            tokens.density.ultra_compact_right_drawer_max_width,
        ),
        (
            "editor.density.compact_bottom_available_height",
            "--compact-bottom-available-height",
            tokens.density.compact_bottom_available_height,
        ),
        (
            "editor.density.compact_bottom_max_height",
            "--compact-bottom-max-height",
            tokens.density.compact_bottom_max_height,
        ),
        (
            "editor.density.compact_bottom_max_available_fraction",
            "--compact-bottom-max-available-fraction",
            tokens.density.compact_bottom_max_available_fraction,
        ),
        (
            "editor.density.compact_bottom_min_height",
            "--compact-bottom-min-height",
            tokens.density.compact_bottom_min_height,
        ),
        (
            "editor.density.ultra_compact_bottom_available_height",
            "--ultra-compact-bottom-available-height",
            tokens.density.ultra_compact_bottom_available_height,
        ),
        (
            "editor.density.ultra_compact_bottom_max_height",
            "--ultra-compact-bottom-max-height",
            tokens.density.ultra_compact_bottom_max_height,
        ),
        (
            "editor.density.ultra_compact_bottom_max_available_fraction",
            "--ultra-compact-bottom-max-available-fraction",
            tokens.density.ultra_compact_bottom_max_available_fraction,
        ),
        (
            "editor.density.ultra_compact_bottom_min_height",
            "--ultra-compact-bottom-min-height",
            tokens.density.ultra_compact_bottom_min_height,
        ),
        (
            "editor.density.minimum_window_width",
            "--minimum-window-width",
            tokens.density.minimum_window_width,
        ),
        (
            "editor.density.minimum_window_height",
            "--minimum-window-height",
            tokens.density.minimum_window_height,
        ),
        (
            "editor.density.ultra_minimum_window_width",
            "--ultra-minimum-window-width",
            tokens.density.ultra_minimum_window_width,
        ),
        (
            "editor.density.ultra_minimum_window_height",
            "--ultra-minimum-window-height",
            tokens.density.ultra_minimum_window_height,
        ),
    ];

    for (canonical_name, legacy_alias, value) in expected {
        assert_eq!(
            tokens.density_value_for_token_name(canonical_name),
            Some(value)
        );
        assert_eq!(
            tokens.density_value_for_token_name(legacy_alias),
            Some(value)
        );
    }
}

#[test]
fn editor_design_tokens_register_canonical_and_css_custom_property_values() {
    let tokens = EditorDesignTokens::workbench_dark();
    let registry = tokens.cascade_token_values();

    assert_eq!(
        registry.get("editor.surface.1"),
        Some(&Value::String("#171a1d".to_string()))
    );
    assert_eq!(
        registry.get("--editor-surface-1"),
        Some(&Value::String("$editor.surface.1".to_string()))
    );
    assert_eq!(
        registry.get("editor.control.height.default"),
        Some(&Value::Float(f64::from(tokens.controls.default_height)))
    );
    assert_eq!(
        registry.get("editor.control.height.large"),
        Some(&Value::Float(f64::from(tokens.controls.large_height)))
    );
    assert_eq!(
        registry.get("editor.density.activity_rail_width"),
        Some(&Value::Float(f64::from(tokens.density.activity_rail_width)))
    );
    assert_eq!(
        registry.get("editor.density.gap.xsmall"),
        Some(&Value::Float(f64::from(tokens.density.gap_xsmall)))
    );
    assert_eq!(
        registry.get("editor.typography.overlay.size"),
        Some(&Value::Float(f64::from(tokens.typography.overlay_size)))
    );
    assert_eq!(
        registry.get("editor.typography.medium.weight"),
        Some(&Value::Integer(i64::from(tokens.typography.medium_weight)))
    );
    assert_eq!(
        registry.get("editor.typography.emphasis.weight"),
        Some(&Value::Integer(i64::from(
            tokens.typography.emphasis_weight
        )))
    );
    assert_eq!(
        registry.get("--editor-control-height-default"),
        Some(&Value::String("$editor.control.height.default".to_string()))
    );
    assert_eq!(
        registry.get("--editor-control-height-large"),
        Some(&Value::String("$editor.control.height.large".to_string()))
    );
    assert_eq!(
        registry.get("editor.state.focused"),
        Some(&Value::String("$editor.surface.selected".to_string()))
    );
    assert_eq!(
        registry.get("--editor-state-focused"),
        Some(&Value::String("$editor.state.focused".to_string()))
    );
    assert_eq!(
        registry.get("--left-drawer-width"),
        Some(&Value::String(
            "$editor.density.left_drawer_width".to_string()
        ))
    );

    let canonical_token_count = registry
        .keys()
        .filter(|name| !name.starts_with("--"))
        .count();
    assert_eq!(canonical_token_count, 94);
    assert_eq!(registry.len(), canonical_token_count * 2 + 26);

    for token_name in registry.keys().filter(|name| !name.starts_with("--")) {
        let custom_property_name = format!("--{}", token_name.replace('.', "-"));
        assert_eq!(
            registry.get(&custom_property_name),
            Some(&Value::String(format!("${token_name}"))),
            "missing mechanical custom-property alias for {token_name}"
        );
    }
}

#[test]
fn editor_design_tokens_resolve_state_roles_without_changing_selector_priority() {
    let tokens = EditorDesignTokens::workbench_dark();

    assert_eq!(
        tokens
            .state_roles
            .role_for_state(UiPainterResolvedState::Selected),
        EditorStateColorRole::SurfaceSelected
    );
    assert_eq!(
        tokens
            .state_roles
            .role_for_state(UiPainterResolvedState::Focused),
        EditorStateColorRole::SurfaceSelected
    );
    assert_eq!(
        tokens.color_for_state(UiPainterResolvedState::Disabled),
        tokens.palette.text_disabled
    );
    assert_eq!(
        tokens.color_for_state(UiPainterResolvedState::Hovered),
        tokens.palette.surface[2]
    );
}

#[test]
fn editor_design_tokens_project_into_theme_document_without_losing_contract_values() {
    let tokens = EditorDesignTokens::workbench_dark();
    let theme = tokens.to_theme_document();

    assert_eq!(theme.id, EDITOR_WORKBENCH_TOKENS_ID);
    assert_eq!(theme.palette.surface, tokens.palette.surface);
    assert_eq!(theme.palette.accent, tokens.palette.accent);
    assert_eq!(theme.palette.separator, tokens.palette.border);
    assert_eq!(theme.control_sizes.default_height, 32.0);
    assert_eq!(theme.control_sizes.compact_height, 30.0);
    assert_eq!(theme.control_sizes.dense_height, 28.0);
    assert_eq!(
        theme.spacing,
        vec![
            0.0,
            tokens.density.gap_xsmall,
            tokens.density.gap_small,
            tokens.density.gap_medium,
            tokens.density.gap_large,
            tokens.density.drawer_padding,
            tokens.density.panel_padding,
        ]
    );
    assert_eq!(theme.shape.radius_panel, tokens.controls.panel_radius);
    assert!(theme
        .typography
        .iter()
        .any(|variant| variant.variant == "body" && variant.family == "system-ui"));
    assert!(theme
        .typography
        .iter()
        .any(|variant| variant.variant == "code" && variant.family == "monospace"));
    assert!(theme.typography.iter().any(|variant| {
        variant.variant == "overlay"
            && variant.size == tokens.typography.overlay_size
            && variant.weight == tokens.typography.emphasis_weight
    }));
}

#[test]
fn editor_design_tokens_feed_painter_styles_through_selector_state() {
    let tokens = EditorDesignTokens::workbench_dark();

    let focused_button = tokens.resolve_painter_style(
        UiPainterState {
            hovered: true,
            selected: true,
            ..UiPainterState::normal()
        },
        UiPainterFamily::Button,
    );
    assert_eq!(focused_button.state, UiPainterResolvedState::Focused);
    assert_eq!(
        focused_button.background_color,
        tokens.palette.surface_selected
    );
    assert_eq!(focused_button.foreground_color, tokens.palette.text_primary);
    assert_eq!(focused_button.border_color, tokens.palette.accent);
    assert_eq!(focused_button.border_width, tokens.controls.border_width);
    assert_eq!(focused_button.corner_radius, tokens.controls.control_radius);
    assert_eq!(
        focused_button.control_height,
        tokens.controls.default_height
    );

    let hovered_icon = tokens.resolve_painter_style(
        UiPainterState {
            hovered: true,
            ..UiPainterState::normal()
        },
        UiPainterFamily::IconButton,
    );
    assert_eq!(hovered_icon.state, UiPainterResolvedState::Hovered);
    assert_eq!(hovered_icon.background_color, tokens.palette.surface[2]);
    assert_eq!(hovered_icon.foreground_color, tokens.palette.text_primary);
    assert_eq!(hovered_icon.corner_radius, tokens.controls.small_radius);

    let disabled_tab = tokens.resolve_painter_style(
        UiPainterState {
            disabled: true,
            hovered: true,
            selected: true,
            ..UiPainterState::normal()
        },
        UiPainterFamily::Tab,
    );
    assert_eq!(disabled_tab.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled_tab.background_color, tokens.palette.surface[1]);
    assert_eq!(disabled_tab.foreground_color, tokens.palette.text_disabled);
    assert_eq!(disabled_tab.border_color, tokens.palette.border);
}
