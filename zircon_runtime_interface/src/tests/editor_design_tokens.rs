use crate::ui::design_tokens::{
    EditorDesignTokens, EditorStateColorRole, EDITOR_WORKBENCH_TOKENS_ID,
};
use crate::ui::style::{UiPainterFamily, UiPainterResolvedState, UiPainterState, UiRgbaColor};

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
    assert_eq!(tokens.controls.border_width, 1.0);
    assert_eq!(tokens.controls.compact_height, 32.0);
    assert_eq!(tokens.controls.dense_height, 28.0);
    assert_eq!(tokens.density.gap_small, 4.0);
    assert_eq!(tokens.density.drawer_padding, 12.0);
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
    assert_eq!(tokens.density_value_for_token_name("--unknown"), None);
}

#[test]
fn editor_design_tokens_resolve_state_roles_without_changing_selector_priority() {
    let tokens = EditorDesignTokens::workbench_dark();

    assert_eq!(
        tokens
            .state_roles
            .role_for_state(UiPainterResolvedState::Selected),
        EditorStateColorRole::Accent
    );
    assert_eq!(
        tokens
            .state_roles
            .role_for_state(UiPainterResolvedState::Focused),
        EditorStateColorRole::Accent
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
    assert_eq!(theme.control_sizes.compact_height, 32.0);
    assert_eq!(theme.control_sizes.dense_height, 28.0);
    assert_eq!(theme.shape.radius_panel, tokens.controls.panel_radius);
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
    assert_eq!(focused_button.background_color, tokens.palette.accent);
    assert_eq!(focused_button.foreground_color, tokens.palette.surface[0]);
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
