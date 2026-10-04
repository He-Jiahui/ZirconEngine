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
fn pressed_warning_alert_preserves_its_status_tone_border() {
    let mut node = TemplatePaneNodeData::default();
    node.pressed = true;

    let style = select_workbench_alert_style(&node, WorkbenchAlertTone::Warning);

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.border, PALETTE.warning);
    assert_ne!(style.border, PALETTE.focus_ring);
}

#[test]
fn alert_dynamic_states_ignore_normal_declared_chrome() {
    let normal = declared_warning_alert();
    let normal_style = select_workbench_alert_style(&normal, WorkbenchAlertTone::Warning);

    assert_eq!(normal_style.surface, [81, 88, 94, 255]);
    assert_eq!(normal_style.border, [109, 116, 122, 255]);
    assert_eq!(normal_style.text, [221, 226, 230, 255]);

    let mut hovered = declared_warning_alert();
    hovered.hovered = true;
    assert_alert_chrome_matches_central_style(&hovered);

    let mut focused = declared_warning_alert();
    focused.focused = true;
    assert_alert_chrome_matches_central_style(&focused);

    let mut pressed = declared_warning_alert();
    pressed.pressed = true;
    assert_alert_chrome_matches_central_style(&pressed);

    let mut selected = declared_warning_alert();
    selected.selected = true;
    assert_alert_chrome_matches_central_style(&selected);

    let mut checked = declared_warning_alert();
    checked.checked = true;
    assert_alert_chrome_matches_central_style(&checked);
}

fn declared_warning_alert() -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(81, 88, 94, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(109, 116, 122, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(221, 226, 230, 255)));
    node
}

fn assert_alert_chrome_matches_central_style(node: &TemplatePaneNodeData) {
    let actual = select_workbench_alert_style(node, WorkbenchAlertTone::Warning);
    let mut central_node = node.clone();
    central_node.button_style.element.background_color = None;
    central_node.button_style.element.border_color = None;
    central_node.button_style.element.foreground_color = None;
    let expected = select_workbench_alert_style(&central_node, WorkbenchAlertTone::Warning);

    assert_eq!(actual.state, expected.state);
    assert_eq!(actual.surface, expected.surface);
    assert_eq!(actual.border, expected.border);
    assert_eq!(actual.text, expected.text);
}
