use super::super::super::super::paint_theme::PALETTE;
use super::*;

fn row() -> TemplatePaneOptionData {
    TemplatePaneOptionData::default()
}

#[test]
fn palette_projects_each_notification_role_from_the_host_theme() {
    let mut host = PALETTE;
    host.popup = [1, 2, 3, 4];
    host.border = [5, 6, 7, 8];
    host.text = [9, 10, 11, 12];
    host.text_muted = [13, 14, 15, 16];
    host.surface_inset = [17, 18, 19, 20];
    host.accent_soft = [21, 22, 23, 24];
    host.surface_disabled = [25, 26, 27, 28];
    host.focus_ring = [29, 30, 31, 32];
    host.accent = [33, 34, 35, 36];
    host.error = [37, 38, 39, 40];
    host.success = [41, 42, 43, 44];
    host.warning = [45, 46, 47, 48];

    let palette = notification_center_palette_from_host(host);

    assert_eq!(palette.panel_surface, [1, 2, 3, 4]);
    assert_eq!(palette.panel_border, [5, 6, 7, 8]);
    assert_eq!(palette.header_text, [9, 10, 11, 12]);
    assert_eq!(palette.muted_text, [13, 14, 15, 16]);
    assert_eq!(palette.row_surface, [17, 18, 19, 20]);
    assert_eq!(palette.row_unread_surface, [21, 22, 23, 24]);
    assert_eq!(palette.row_disabled_surface, [25, 26, 27, 28]);
    assert_eq!(palette.row_border, [5, 6, 7, 8]);
    assert_eq!(palette.row_focus_border, [29, 30, 31, 32]);
    assert_eq!(palette.accent, [33, 34, 35, 36]);
    assert_eq!(palette.error, [37, 38, 39, 40]);
    assert_eq!(palette.success, [41, 42, 43, 44]);
    assert_eq!(palette.warning, [45, 46, 47, 48]);
}

#[test]
fn notification_row_state_priority_keeps_disabled_and_selected_explicit() {
    let palette = current_notification_center_palette();
    let disabled = TemplatePaneOptionData {
        disabled: true,
        unread: true,
        focused: true,
        ..row()
    };
    let selected = TemplatePaneOptionData {
        selected: true,
        focused: true,
        ..row()
    };

    assert_eq!(
        row_background(&disabled, palette),
        palette.row_disabled_surface
    );
    assert_eq!(row_border(&selected, palette), palette.accent);
    assert_eq!(
        severity_color("warning", palette),
        palette.warning,
        "semantic tones must not use local RGB literals"
    );
}
