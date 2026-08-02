use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{PALETTE, project_host_palette};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;
use zircon_runtime_interface::ui::style::UiRgbaColor;

#[test]
fn status_signal_unavailable_states_mute_icon_and_text() {
    let mut disabled = TemplatePaneNodeData::default();
    disabled.disabled = true;
    disabled.hovered = true;
    disabled.label_color = Color::from_rgb_u8(242, 195, 86);
    disabled.value_color = Color::from_rgb_u8(135, 146, 153);

    let disabled_style =
        select_workbench_status_signal_style(&disabled, WorkbenchStatusSignalKind::Warning);
    assert_eq!(disabled_style.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled_style.icon_fill, PALETTE.text_disabled);
    assert_eq!(disabled_style.text, PALETTE.text_disabled);

    let mut loading = TemplatePaneNodeData::default();
    loading.hovered = true;
    loading.button_style.loading = true;
    loading.label_color = Color::from_rgb_u8(88, 184, 102);
    loading.value_color = Color::from_rgb_u8(143, 154, 160);

    let loading_style =
        select_workbench_status_signal_style(&loading, WorkbenchStatusSignalKind::Success);
    assert_eq!(loading_style.state, UiPainterResolvedState::Loading);
    assert_eq!(loading_style.icon_fill, PALETTE.text_disabled);
    assert_eq!(loading_style.text, PALETTE.text_disabled);
}

#[test]
fn status_control_palette_projects_from_host_appearance_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface_disabled = UiRgbaColor::from_u8(8, 9, 10, 255);
    tokens.palette.surface_hover = UiRgbaColor::from_u8(18, 19, 20, 255);
    tokens.palette.surface_selected = UiRgbaColor::from_u8(28, 29, 30, 255);
    tokens.palette.border_disabled = UiRgbaColor::from_u8(38, 39, 40, 255);
    tokens.palette.focus_ring = UiRgbaColor::from_u8(48, 49, 50, 255);
    tokens.palette.text_primary = UiRgbaColor::from_u8(58, 59, 60, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(68, 69, 70, 255);
    tokens.palette.text_disabled = UiRgbaColor::from_u8(78, 79, 80, 255);
    tokens.palette.success = UiRgbaColor::from_u8(88, 89, 90, 255);
    tokens.palette.warning = UiRgbaColor::from_u8(98, 99, 100, 255);
    tokens.palette.info = UiRgbaColor::from_u8(108, 109, 110, 255);

    let host = project_host_palette(&tokens);
    let palette = palette::workbench_status_control_palette_from_host(host);

    assert_eq!(palette.surface_disabled, [8, 9, 10, 255]);
    assert_eq!(palette.surface_hover, [18, 19, 20, 255]);
    assert_eq!(palette.surface_selected, [28, 29, 30, 255]);
    assert_eq!(palette.border_disabled, [38, 39, 40, 255]);
    assert_eq!(palette.focus_ring, [48, 49, 50, 255]);
    assert_eq!(palette.text, [58, 59, 60, 255]);
    assert_eq!(palette.text_muted, [68, 69, 70, 255]);
    assert_eq!(palette.text_disabled, [78, 79, 80, 255]);
    assert_eq!(palette.success, [88, 89, 90, 255]);
    assert_eq!(palette.warning, [98, 99, 100, 255]);
    assert_eq!(palette.info, [108, 109, 110, 255]);
    assert_eq!(palette.icon_color, palette.text_muted);
    assert_eq!(palette.icon_muted, palette.text_disabled);
    assert_eq!(palette.no_errors_fill, palette.success);
}
