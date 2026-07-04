use super::palette::{workbench_dropdown_palette, workbench_dropdown_palette_from_host};
use super::selection::select_workbench_dropdown_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::project_host_palette;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn dropdown_palette_projects_from_host_appearance_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface_recessed = UiRgbaColor::from_u8(6, 8, 10, 255);
    tokens.palette.popup = UiRgbaColor::from_u8(12, 14, 16, 255);
    tokens.palette.surface[3] = UiRgbaColor::from_u8(18, 20, 22, 255);
    tokens.palette.surface_disabled = UiRgbaColor::from_u8(24, 26, 28, 255);
    tokens.palette.border = UiRgbaColor::from_u8(30, 32, 34, 255);
    tokens.palette.focus_ring = UiRgbaColor::from_u8(42, 180, 210, 255);
    tokens.palette.border_disabled = UiRgbaColor::from_u8(50, 52, 54, 255);
    tokens.palette.error = UiRgbaColor::from_u8(220, 84, 74, 255);
    tokens.palette.text_primary = UiRgbaColor::from_u8(230, 234, 238, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(144, 152, 160, 255);
    tokens.palette.text_disabled = UiRgbaColor::from_u8(92, 100, 108, 255);

    let palette = workbench_dropdown_palette_from_host(project_host_palette(&tokens));

    assert_eq!(palette.surface, [6, 8, 10, 255]);
    assert_eq!(palette.hover_surface, [12, 14, 16, 255]);
    assert_eq!(palette.open_surface, [18, 20, 22, 255]);
    assert_eq!(palette.disabled_surface, [24, 26, 28, 255]);
    assert_eq!(palette.border, [30, 32, 34, 255]);
    assert_eq!(palette.focus_border, [42, 180, 210, 255]);
    assert_eq!(palette.hover_border, [30, 32, 34, 255]);
    assert_eq!(palette.disabled_border, [50, 52, 54, 255]);
    assert_eq!(palette.error_border, [220, 84, 74, 255]);
    assert_eq!(palette.text, [230, 234, 238, 255]);
    assert_eq!(palette.placeholder, [92, 100, 108, 255]);
    assert_eq!(palette.disabled_text, [92, 100, 108, 255]);
    assert_eq!(palette.chevron, [144, 152, 160, 255]);
    assert_eq!(palette.active_chevron, [42, 180, 210, 255]);
}

#[test]
fn dropdown_loading_state_uses_unavailable_visuals() {
    let palette = workbench_dropdown_palette();
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.popup_open = true;
    node.selected = true;
    node.validation_level = "danger".into();
    node.button_style.loading = true;
    node.label_brightness = 1.8;
    node.value_color = Color::from_rgb_u8(205, 216, 221);
    node.icon_color = Color::from_rgb_u8(128, 234, 255);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(15, 101, 116, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

    let style = select_workbench_dropdown_style(&node, false);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, palette.disabled_surface);
    assert_eq!(style.border, palette.disabled_border);
    assert_eq!(style.text, palette.disabled_text);
    assert_eq!(style.chevron, palette.disabled_text);
}
