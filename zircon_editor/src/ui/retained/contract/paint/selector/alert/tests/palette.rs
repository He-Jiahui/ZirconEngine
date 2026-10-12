use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn alert_tone_style_projects_info_from_host_palette() {
    let mut palette = PALETTE;
    palette.info_container = [10, 11, 12, 255];
    palette.info = [13, 14, 15, 255];
    palette.text = [16, 17, 18, 255];

    let style = alert_tone_style_from_host(
        WorkbenchAlertTone::Info,
        UiPainterResolvedState::Normal,
        palette,
    );

    assert_eq!(style.surface, [10, 11, 12, 255]);
    assert_eq!(style.border, [13, 14, 15, 255]);
    assert_eq!(style.mark, [13, 14, 15, 255]);
    assert_eq!(style.text, [16, 17, 18, 255]);
}

#[test]
fn alert_tone_style_projects_status_tones_from_host_palette() {
    let mut palette = PALETTE;
    palette.success_container = [20, 21, 22, 255];
    palette.success = [23, 24, 25, 255];
    palette.warning_container = [30, 31, 32, 255];
    palette.warning = [33, 34, 35, 255];
    palette.error_container = [40, 41, 42, 255];
    palette.error = [43, 44, 45, 255];
    palette.text = [46, 47, 48, 255];

    let success = alert_tone_style_from_host(
        WorkbenchAlertTone::Success,
        UiPainterResolvedState::Normal,
        palette,
    );
    let warning = alert_tone_style_from_host(
        WorkbenchAlertTone::Warning,
        UiPainterResolvedState::Normal,
        palette,
    );
    let error = alert_tone_style_from_host(
        WorkbenchAlertTone::Error,
        UiPainterResolvedState::Normal,
        palette,
    );

    assert_eq!(success.surface, [20, 21, 22, 255]);
    assert_eq!(success.border, [23, 24, 25, 255]);
    assert_eq!(success.mark, [23, 24, 25, 255]);
    assert_eq!(success.text, [46, 47, 48, 255]);
    assert_eq!(warning.surface, [30, 31, 32, 255]);
    assert_eq!(warning.border, [33, 34, 35, 255]);
    assert_eq!(warning.mark, [33, 34, 35, 255]);
    assert_eq!(warning.text, [46, 47, 48, 255]);
    assert_eq!(error.surface, [40, 41, 42, 255]);
    assert_eq!(error.border, [43, 44, 45, 255]);
    assert_eq!(error.mark, [43, 44, 45, 255]);
    assert_eq!(error.text, [46, 47, 48, 255]);
}

#[test]
fn alert_palette_projects_state_roles_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_disabled = [50, 51, 52, 255];
    palette.border_disabled = [53, 54, 55, 255];
    palette.text_disabled = [56, 57, 58, 255];
    palette.text = [62, 63, 64, 255];

    let alert_palette = workbench_alert_palette_from_host(palette);

    assert_eq!(alert_palette.disabled_surface, [50, 51, 52, 255]);
    assert_eq!(alert_palette.disabled_border, [53, 54, 55, 255]);
    assert_eq!(alert_palette.disabled_text, [56, 57, 58, 255]);
    assert_eq!(alert_palette.text, [62, 63, 64, 255]);
}
