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
fn text_field_focused_state_uses_starship_primary_outline_without_raising_input_surface() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_text_field_style(&node, false);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.surface, PALETTE.surface_inset);
    assert_eq!(style.border, PALETTE.focus_ring);
}

#[test]
fn named_showcase_field_honors_known_hidden_runtime_focus() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchInputFocused".into(),
        focused: true,
        focus_visible_known: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_text_field_style(&node, false);

    assert_eq!(style.state, UiPainterResolvedState::Normal);
    assert_eq!(style.surface, PALETTE.surface_inset);
    assert_eq!(style.border, PALETTE.separator_soft);
    assert_ne!(style.border, PALETTE.focus_ring);
}

#[test]
fn text_field_hover_uses_hover_outline_without_impersonating_keyboard_focus() {
    let node = TemplatePaneNodeData {
        hovered: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_text_field_style(&node, false);

    assert_eq!(style.state, UiPainterResolvedState::Hovered);
    assert_eq!(style.surface, PALETTE.surface_inset);
    assert_eq!(style.border, PALETTE.surface_hover);
    assert_ne!(style.border, PALETTE.focus_ring);
}

#[test]
fn text_field_dynamic_outlines_ignore_normal_style_overrides() {
    let override_surface = UiStyleColor::Rgba(UiRgbaColor::from_u8(93, 97, 101, 255));
    let override_border = UiStyleColor::Rgba(UiRgbaColor::from_u8(106, 111, 116, 255));

    let mut normal = TemplatePaneNodeData::default();
    normal.button_style.element.background_color = Some(override_surface.clone());
    normal.button_style.element.border_color = Some(override_border.clone());
    let normal_style = select_workbench_text_field_style(&normal, false);

    let mut hovered = TemplatePaneNodeData::default();
    hovered.hovered = true;
    hovered.button_style.element.background_color = Some(override_surface.clone());
    hovered.button_style.element.border_color = Some(override_border.clone());
    let hovered_style = select_workbench_text_field_style(&hovered, false);

    let mut focused = TemplatePaneNodeData::default();
    focused.focused = true;
    focused.button_style.element.background_color = Some(override_surface.clone());
    focused.button_style.element.border_color = Some(override_border.clone());
    let focused_style = select_workbench_text_field_style(&focused, false);

    let mut pressed = TemplatePaneNodeData::default();
    pressed.pressed = true;
    pressed.button_style.element.background_color = Some(override_surface.clone());
    pressed.button_style.element.border_color = Some(override_border.clone());
    let pressed_style = select_workbench_text_field_style(&pressed, false);

    let mut selected = TemplatePaneNodeData::default();
    selected.selected = true;
    selected.button_style.element.background_color = Some(override_surface.clone());
    selected.button_style.element.border_color = Some(override_border.clone());
    let selected_style = select_workbench_text_field_style(&selected, false);

    let mut checked = TemplatePaneNodeData::default();
    checked.checked = true;
    checked.button_style.element.background_color = Some(override_surface.clone());
    checked.button_style.element.border_color = Some(override_border.clone());
    let checked_style = select_workbench_text_field_style(&checked, false);

    let mut invalid = TemplatePaneNodeData::default();
    invalid.validation_level = "error".into();
    invalid.button_style.element.background_color = Some(override_surface);
    invalid.button_style.element.border_color = Some(override_border);
    let invalid_style = select_workbench_text_field_style(&invalid, false);

    assert_eq!(normal_style.surface, [93, 97, 101, 255]);
    assert_eq!(normal_style.border, [106, 111, 116, 255]);
    assert_eq!(hovered_style.surface, PALETTE.surface_inset);
    assert_eq!(hovered_style.border, PALETTE.surface_hover);
    assert_eq!(focused_style.surface, PALETTE.surface_inset);
    assert_eq!(focused_style.border, PALETTE.focus_ring);
    assert_eq!(pressed_style.surface, PALETTE.surface_inset);
    assert_eq!(pressed_style.border, PALETTE.surface_hover);
    assert_ne!(pressed_style.border, PALETTE.focus_ring);
    assert_eq!(selected_style.surface, PALETTE.surface_inset);
    assert_eq!(selected_style.border, PALETTE.separator_soft);
    assert_eq!(checked_style.surface, PALETTE.surface_inset);
    assert_eq!(checked_style.border, PALETTE.separator_soft);
    assert_eq!(invalid_style.surface, PALETTE.surface_inset);
    assert_eq!(invalid_style.border, PALETTE.error);
}

#[test]
fn text_field_palette_projects_from_global_appearance_palette() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface_recessed = UiRgbaColor::from_u8(11, 15, 18, 255);
    tokens.palette.surface_hover = UiRgbaColor::from_u8(31, 37, 42, 255);
    tokens.palette.focus_ring = UiRgbaColor::from_u8(9, 180, 220, 255);
    tokens.palette.border = UiRgbaColor::from_u8(72, 82, 90, 255);
    tokens.palette.separator_soft = UiRgbaColor::from_u8(42, 48, 54, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(150, 160, 168, 255);

    let palette =
        super::palette::workbench_text_field_palette_from_host(project_host_palette(&tokens));

    assert_eq!(palette.surface, [11, 15, 18, 255]);
    assert_eq!(palette.toolbar_surface, [11, 15, 18, 255]);
    assert_eq!(palette.hover_surface, [11, 15, 18, 255]);
    assert_eq!(palette.hover_border, [31, 37, 42, 255]);
    assert_eq!(palette.focus_border, [9, 180, 220, 255]);
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

    assert_eq!(style.surface, PALETTE.surface_inset);
    assert_eq!(style.border, PALETTE.separator_soft);
    assert_ne!(style.surface, [1, 2, 3, 255]);
    assert_ne!(style.border, [4, 5, 6, 255]);
}
