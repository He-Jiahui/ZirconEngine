use super::super::palette::workbench_alert_palette_from_host;
use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn alert_unavailable_state_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_disabled = [10, 11, 12, 255];
    palette.border_disabled = [13, 14, 15, 255];
    palette.text_disabled = [16, 17, 18, 255];

    let style = alert_state_style_from_palette(
        WorkbenchAlertTone::Warning,
        UiPainterResolvedState::Loading,
        workbench_alert_palette_from_host(palette),
    );

    assert_eq!(style.surface, [10, 11, 12, 255]);
    assert_eq!(style.border, [13, 14, 15, 255]);
    assert_eq!(style.mark, [16, 17, 18, 255]);
    assert_eq!(style.text, [16, 17, 18, 255]);
}

#[test]
fn alert_pressed_state_preserves_status_tone_border_from_host_palette() {
    let mut palette = PALETTE;
    palette.warning_container = [20, 21, 22, 255];
    palette.warning = [23, 24, 25, 255];

    let style = alert_state_style_from_palette(
        WorkbenchAlertTone::Warning,
        UiPainterResolvedState::Pressed,
        workbench_alert_palette_from_host(palette),
    );

    assert_eq!(style.surface, [20, 21, 22, 255]);
    assert_eq!(style.border, [23, 24, 25, 255]);
    assert_eq!(style.mark, [23, 24, 25, 255]);
    // BUG: [CR-EDITOR-PAINT-STYLE-0001] 此断言把正文误当 warning 色；夹具未改 text，实际正文仍为 [232, 232, 232, 255]。
    assert_eq!(style.text, [23, 24, 25, 255]);
}

#[test]
fn alert_focused_state_keeps_tone_border_from_host_palette() {
    let mut palette = PALETTE;
    palette.warning_container = [30, 31, 32, 255];
    palette.warning = [33, 34, 35, 255];
    palette.focus_ring = [36, 37, 38, 255];

    let style = alert_state_style_from_palette(
        WorkbenchAlertTone::Warning,
        UiPainterResolvedState::Focused,
        workbench_alert_palette_from_host(palette),
    );

    assert_eq!(style.surface, [30, 31, 32, 255]);
    assert_eq!(style.border, [33, 34, 35, 255]);
    assert_ne!(style.border, [36, 37, 38, 255]);
}
