use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn alert_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.button_style.loading = true;
    node.icon_color = Color::from_rgb_u8(224, 163, 58);
    node.label_color = Color::from_rgb_u8(208, 217, 221);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(69, 50, 20, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(132, 94, 35, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(224, 163, 58, 255)));

    let style = select_workbench_alert_style(&node, WorkbenchAlertTone::Warning);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, PALETTE.surface_disabled);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.mark, PALETTE.text_disabled);
    assert_eq!(style.text, PALETTE.text_disabled);
}

#[test]
fn focused_warning_alert_keeps_tone_border_without_active_focus_ring() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_alert_style(&node, WorkbenchAlertTone::Warning);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.border, PALETTE.warning);
    assert_ne!(style.border, PALETTE.focus_ring);
}

#[test]
fn pressed_warning_alert_still_uses_active_focus_ring_border() {
    let mut node = TemplatePaneNodeData::default();
    node.pressed = true;

    let style = select_workbench_alert_style(&node, WorkbenchAlertTone::Warning);

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.border, PALETTE.focus_ring);
}
