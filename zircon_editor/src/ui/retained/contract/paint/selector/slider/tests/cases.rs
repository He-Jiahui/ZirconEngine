use super::palette::{
    workbench_slider_palette, workbench_slider_palette_from_host, WORKBENCH_SLIDER_TRACK_DISABLED,
};
use super::selection::select_workbench_slider_style;
use super::state::is_workbench_slider_state_hot;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{project_host_palette, PALETTE};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn slider_palette_projects_from_host_appearance_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface_recessed = UiRgbaColor::from_u8(6, 8, 10, 255);
    tokens.palette.border_disabled = UiRgbaColor::from_u8(34, 40, 46, 255);
    tokens.palette.separator_strong = UiRgbaColor::from_u8(70, 80, 90, 255);
    tokens.palette.separator_soft = UiRgbaColor::from_u8(31, 37, 43, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(142, 150, 158, 255);
    tokens.palette.text_primary = UiRgbaColor::from_u8(225, 230, 234, 255);
    tokens.palette.popup = UiRgbaColor::from_u8(12, 14, 16, 255);
    tokens.palette.border = UiRgbaColor::from_u8(48, 56, 64, 255);
    tokens.palette.text_disabled = UiRgbaColor::from_u8(92, 100, 108, 255);
    tokens.palette.focus_ring = UiRgbaColor::from_u8(58, 180, 198, 255);

    let palette = workbench_slider_palette_from_host(project_host_palette(&tokens));

    assert_eq!(palette.track, [6, 8, 10, 255]);
    assert_eq!(palette.track_disabled, [34, 40, 46, 255]);
    assert_eq!(palette.fill, [70, 80, 90, 255]);
    assert_eq!(palette.tick, [31, 37, 43, 255]);
    assert_eq!(palette.label_text, [142, 150, 158, 255]);
    assert_eq!(palette.value_text, [142, 150, 158, 255]);
    assert_eq!(palette.thumb, [225, 230, 234, 255]);
    assert_eq!(palette.thumb_halo, [58, 180, 198, 26]);
    assert_eq!(palette.value_surface, [12, 14, 16, 255]);
    assert_eq!(palette.value_border, [48, 56, 64, 255]);
    assert_eq!(palette.text_disabled, [92, 100, 108, 255]);
}

#[test]
fn slider_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.pressed = true;
    node.drop_hovered = true;
    node.button_style.loading = true;
    node.validation_level = "warning".into();
    node.value_color = Color::from_rgb_u8(53, 199, 208);
    node.icon_color = Color::from_rgb_u8(201, 242, 246);
    node.label_color = Color::from_rgb_u8(174, 189, 196);
    node.state_layer_color = Color::from_argb_u8(58, 53, 199, 208);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(54, 64, 70, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(53, 199, 208, 255)));

    let style = select_workbench_slider_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.track, WORKBENCH_SLIDER_TRACK_DISABLED);
    assert_eq!(style.fill, PALETTE.text_disabled);
    assert_eq!(style.thumb, PALETTE.text_disabled);
    assert_eq!(style.thumb_outline, PALETTE.border_disabled);
    assert_eq!(style.thumb_halo, None);
    assert_eq!(style.value_surface, PALETTE.surface_disabled);
    assert_eq!(style.value_border, PALETTE.border_disabled);
    assert_eq!(style.range_value_border, PALETTE.border_disabled);
    assert_eq!(style.label_text, PALETTE.text_disabled);
    assert_eq!(style.value_text, PALETTE.text_disabled);
    assert_eq!(style.tick, PALETTE.border_disabled);
}

#[test]
fn focused_slider_keeps_neutral_value_border_with_focus_halo() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_slider_style(&node);
    let palette = workbench_slider_palette();

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert!(!is_workbench_slider_state_hot(style.state));
    assert_eq!(style.value_border, palette.value_border);
    assert_eq!(style.thumb_halo, Some(palette.thumb_halo));
}

#[test]
fn pressed_slider_keeps_active_value_border_and_hot_halo() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.pressed = true;

    let style = select_workbench_slider_style(&node);
    let palette = workbench_slider_palette();

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert!(is_workbench_slider_state_hot(style.state));
    assert_eq!(style.value_border, style.fill);
    assert_eq!(style.thumb_halo, Some(palette.thumb_halo));
}
