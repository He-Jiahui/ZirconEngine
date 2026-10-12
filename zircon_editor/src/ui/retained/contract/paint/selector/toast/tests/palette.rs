use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn toast_normal_style_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent_soft = [10, 11, 12, 247];
    palette.border = [13, 14, 15, 20];
    palette.text = [16, 17, 18, 255];
    palette.accent = [19, 20, 21, 255];
    palette.text_muted = [22, 23, 24, 255];

    let style = toast_normal_style_from_host(UiPainterResolvedState::Normal, palette);

    assert_eq!(style.surface, [10, 11, 12, 247]);
    assert_eq!(style.border, [13, 14, 15, 20]);
    assert_eq!(style.text, [16, 17, 18, 255]);
    assert_eq!(style.mark, [19, 20, 21, 255]);
    assert_eq!(style.action, [19, 20, 21, 255]);
    assert_eq!(style.close, [22, 23, 24, 255]);
}

#[test]
fn toast_interaction_surfaces_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_selected = [30, 31, 32, 255];
    palette.surface_pressed = [33, 34, 35, 255];

    assert_eq!(toast_hover_surface_from_host(palette), [30, 31, 32, 255]);
    assert_eq!(toast_pressed_surface_from_host(palette), [33, 34, 35, 255]);
}

#[test]
fn toast_palette_projects_state_roles_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent_soft = [40, 41, 42, 247];
    palette.focus_ring = [41, 42, 43, 255];
    palette.surface_disabled = [43, 44, 45, 255];
    palette.border_disabled = [46, 47, 48, 255];
    palette.text_disabled = [49, 50, 51, 255];

    let toast_palette = workbench_toast_palette_from_host(palette);

    assert_eq!(toast_palette.hover_border, [40, 41, 42, 247]);
    assert_eq!(toast_palette.focus_border, [41, 42, 43, 255]);
    assert_eq!(toast_palette.disabled_surface, [43, 44, 45, 255]);
    assert_eq!(toast_palette.disabled_border, [46, 47, 48, 255]);
    assert_eq!(toast_palette.disabled_text, [49, 50, 51, 255]);
}
