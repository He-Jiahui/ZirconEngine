use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{project_host_palette, PALETTE};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn text_field_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.validation_level = "error".into();
    node.button_style.loading = true;
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(16, 22, 26, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

    let style = select_workbench_text_field_style(&node, true);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, WORKBENCH_TEXT_FIELD_DISABLED_SURFACE);
    assert_eq!(style.border, WORKBENCH_TEXT_FIELD_DISABLED_BORDER);
    assert_eq!(style.text, WORKBENCH_TEXT_FIELD_DISABLED_TEXT);
    assert_eq!(style.stepper, PALETTE.text_disabled);
    assert_eq!(style.stepper_divider, PALETTE.border_disabled);
}

#[test]
fn text_field_focused_state_uses_neutral_slate_chrome() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_text_field_style(&node, false);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.surface, PALETTE.surface);
    assert_eq!(style.border, PALETTE.border);
    assert_ne!(style.border, PALETTE.accent);
    assert_ne!(style.border, PALETTE.focus_ring);
}

#[test]
fn text_field_palette_projects_from_global_appearance_palette() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface_recessed = UiRgbaColor::from_u8(11, 15, 18, 255);
    tokens.palette.surface[2] = UiRgbaColor::from_u8(31, 37, 42, 255);
    tokens.palette.border = UiRgbaColor::from_u8(72, 82, 90, 255);
    tokens.palette.separator_soft = UiRgbaColor::from_u8(42, 48, 54, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(150, 160, 168, 255);

    let palette =
        super::palette::workbench_text_field_palette_from_host(project_host_palette(&tokens));

    assert_eq!(palette.surface, [11, 15, 18, 255]);
    assert_eq!(palette.toolbar_surface, [31, 37, 42, 255]);
    assert_eq!(palette.hover_surface, [31, 37, 42, 255]);
    assert_eq!(palette.focused_border, [72, 82, 90, 255]);
    assert_eq!(palette.placeholder, [150, 160, 168, 255]);
    assert_eq!(palette.stepper_divider, [42, 48, 54, 255]);
}

#[test]
fn asset_browser_toolbar_search_field_ignores_legacy_declared_chrome() {
    let mut node = TemplatePaneNodeData {
        control_id: "SearchEdited".into(),
        ..TemplatePaneNodeData::default()
    };
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(1, 2, 3, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(4, 5, 6, 255)));

    let style = select_workbench_text_field_style(&node, true);

    assert_eq!(style.surface, PALETTE.surface);
    assert_eq!(style.border, PALETTE.separator_soft);
    assert_ne!(style.surface, [1, 2, 3, 255]);
    assert_ne!(style.border, [4, 5, 6, 255]);
}
