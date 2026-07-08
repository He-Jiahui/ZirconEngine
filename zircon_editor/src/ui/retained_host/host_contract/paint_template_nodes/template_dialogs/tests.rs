use super::identity::DialogKind;
use super::metrics::dialog_metrics_from_host;
use super::style::{dialog_border_color, dialog_palette_from_host};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

#[test]
fn dialog_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.radius_control = 5.0;
    host.border_width = 1.5;
    host.font_body = 11.0;
    host.font_large = 15.0;
    host.line_height_ratio = 1.25;
    host.gap_s = 5.0;
    host.gap_m = 9.0;
    host.gap_l = 13.0;
    host.row_height = 26.0;
    host.selection_indicator_width = 3.0;
    host.text_clip_guard = 4.0;

    let metrics = dialog_metrics_from_host(host);

    assert_eq!(metrics.padding_x, 22.0);
    assert_eq!(metrics.title_top, 21.0);
    assert_eq!(metrics.body_top, 52.0);
    assert_eq!(metrics.title_font_size, 16.5);
    assert_eq!(metrics.title_line_height, 20.625);
    assert_eq!(metrics.body_font_size, 13.75);
    assert_eq!(metrics.body_line_height, 18.6875);
    assert_eq!(metrics.severity_mark_width, 6.0);
    assert_eq!(metrics.radius, 8.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.action_bottom, 22.0);
    assert_eq!(metrics.action_gap, 18.0);
    assert_eq!(metrics.action_min_width, 61.0);
    assert_eq!(metrics.action_text_padding_x, 12.0);
    assert_eq!(metrics.action_text_clip_guard, 4.0);
    assert_eq!(metrics.action_font_size, 13.75);
    assert_eq!(metrics.action_line_height, 18.6875);
}

#[test]
fn dialog_palette_projects_from_host_palette() {
    let mut host = PALETTE;
    host.popup = [10, 11, 12, 255];
    host.border = [20, 21, 22, 255];
    host.focus_ring = [30, 31, 32, 255];
    host.text = [40, 41, 42, 255];
    host.text_muted = [50, 51, 52, 255];
    host.accent = [60, 61, 62, 255];
    host.info = [70, 71, 72, 255];
    host.info_container = [80, 81, 82, 255];
    host.warning = [90, 91, 92, 255];
    host.warning_container = [100, 101, 102, 255];
    host.error = [110, 111, 112, 255];
    host.error_container = [120, 121, 122, 255];
    host.surface_disabled = [130, 131, 132, 255];
    host.border_disabled = [140, 141, 142, 255];
    host.text_disabled = [150, 151, 152, 255];

    let palette = dialog_palette_from_host(host);

    assert_eq!(palette.surface, [10, 11, 12, 255]);
    assert_eq!(palette.border, [20, 21, 22, 255]);
    assert_eq!(palette.active_border, [30, 31, 32, 255]);
    assert_eq!(palette.title, [40, 41, 42, 255]);
    assert_eq!(palette.body, [50, 51, 52, 255]);
    assert_eq!(palette.action, [60, 61, 62, 255]);
    assert_eq!(palette.info, [70, 71, 72, 255]);
    assert_eq!(palette.info_border, [80, 81, 82, 255]);
    assert_eq!(palette.warning, [90, 91, 92, 255]);
    assert_eq!(palette.warning_border, [100, 101, 102, 255]);
    assert_eq!(palette.error, [110, 111, 112, 255]);
    assert_eq!(palette.error_border, [120, 121, 122, 255]);
    assert_eq!(palette.disabled_surface, [130, 131, 132, 255]);
    assert_eq!(palette.disabled_border, [140, 141, 142, 255]);
    assert_eq!(palette.disabled_text, [150, 151, 152, 255]);
}

#[test]
fn focused_dialog_keeps_neutral_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let border = dialog_border_color(&node, DialogKind::Dialog, false);

    assert_eq!(border, PALETTE.border);
    assert_ne!(border, PALETTE.focus_ring);
}

#[test]
fn pressed_dialog_uses_active_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.pressed = true;

    let border = dialog_border_color(&node, DialogKind::Dialog, false);

    assert_eq!(border, PALETTE.focus_ring);
}

#[test]
fn open_dialog_uses_active_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.popup_open = true;

    let border = dialog_border_color(&node, DialogKind::Dialog, false);

    assert_eq!(border, PALETTE.focus_ring);
}
