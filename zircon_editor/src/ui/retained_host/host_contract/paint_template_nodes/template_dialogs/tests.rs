use super::super::render_commands::HostPaintCommandKind;
use super::commands::push_dialog_commands;
use super::identity::{dialog_paint_state, DialogKind, DialogPaintState};
use super::layout::{body_rect, dialog_has_visible_area, pixel_aligned_rect};
use super::metrics::dialog_metrics_from_host;
use super::style::{dialog_border_color, dialog_palette_from_host};
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

#[test]
fn dialog_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.radius_control = 5.0;
    host.border_width = 1.5;
    host.font_small = 8.0;
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
    assert_eq!(metrics.content_gap, 6.5);
    assert_eq!(metrics.content_action_gap, 10.5);
    assert_eq!(metrics.title_font_size, 11.0);
    assert_eq!(metrics.title_line_height, 13.75);
    assert_eq!(metrics.body_font_size, 8.0);
    assert_eq!(metrics.body_line_height, 10.0);
    assert_eq!(metrics.severity_mark_width, 6.0);
    assert_eq!(metrics.radius, 8.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.action_bottom, 10.5);
    assert_eq!(metrics.legacy_action_bottom, 22.0);
    assert_eq!(metrics.action_gap, 18.0);
    assert_eq!(metrics.action_stack_gap, 6.5);
    assert_eq!(metrics.action_min_width, 61.0);
    assert_eq!(metrics.action_height, 26.0);
    assert_eq!(metrics.action_radius, 5.0);
    assert_eq!(metrics.action_text_padding_x, 12.0);
    assert_eq!(metrics.action_text_clip_guard, 4.0);
    assert_eq!(metrics.action_font_size, 11.0);
    assert_eq!(metrics.action_line_height, 13.75);
}

#[test]
fn dialog_typography_uses_unreal_standard_dialog_roles() {
    let metrics = dialog_metrics_from_host(METRICS);

    assert_eq!(metrics.title_font_size, METRICS.font_body);
    assert_eq!(metrics.body_font_size, METRICS.font_small);
    assert_eq!(metrics.action_font_size, METRICS.font_body);
    assert!(metrics.title_font_size < METRICS.font_large);
    assert!(metrics.body_font_size < metrics.title_font_size);
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

#[test]
fn dialog_body_reserves_a_separate_interaction_rail_after_the_title() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 220.0,
        height: 104.0,
    };
    let metrics = super::metrics::dialog_metrics();
    let action_top = 102.0;
    let body = body_rect(&rect, DialogKind::ConfirmDialog, Some(action_top))
        .expect("a standard confirm dialog keeps one body line above its action rail");

    assert!(body.y >= rect.y + metrics.title_top + metrics.title_line_height + metrics.content_gap);
    assert!(body.y + body.height <= action_top - metrics.content_action_gap);
}

#[test]
fn short_confirm_dialog_compacts_the_body_action_gap_before_dropping_content() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 220.0,
        height: 84.0,
    };
    let metrics = super::metrics::dialog_metrics();
    let action_top = rect.y + rect.height - metrics.action_bottom - metrics.action_height;
    let body = body_rect(&rect, DialogKind::ConfirmDialog, Some(action_top))
        .expect("the atlas confirm dialog compacts spacing instead of hiding its body");
    let effective_gap = action_top - (body.y + body.height);

    assert!(effective_gap >= metrics.content_gap);
    assert!(effective_gap < metrics.content_action_gap);
}

#[test]
fn alert_dialog_retains_its_separate_legacy_identity_and_body_offset() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 220.0,
        height: 104.0,
    };
    let mut node = TemplatePaneNodeData::default();
    node.role = "AlertDialog".to_string();
    node.popup_open = true;

    assert_eq!(
        dialog_paint_state(&node),
        DialogPaintState::Open(DialogKind::AlertDialog)
    );
    assert!(DialogKind::AlertDialog.uses_severity_chrome());
    let body = body_rect(&rect, DialogKind::AlertDialog, None)
        .expect("the legacy alert body is always positioned by its established body offset");
    assert_eq!(body.y, rect.y + super::metrics::dialog_metrics().body_top);
}

#[test]
fn collapsed_dialog_extent_stays_collapsed_after_pixel_alignment() {
    let rect = FrameRect {
        x: 12.4,
        y: 18.6,
        width: 0.0,
        height: 0.0,
    };

    let aligned = pixel_aligned_rect(&rect);

    assert_eq!(aligned.width, 0.0);
    assert_eq!(aligned.height, 0.0);
    assert!(!dialog_has_visible_area(&aligned));
}

#[test]
fn dialog_root_must_stay_fully_within_its_clip_before_emitting_commands() {
    let node = TemplatePaneNodeData {
        role: "Dialog".to_string(),
        popup_open: true,
        text: "Discard changes?".to_string(),
        ..TemplatePaneNodeData::default()
    };
    let root = FrameRect {
        x: 8.0,
        y: 20.0,
        width: 180.0,
        height: 120.0,
    };
    let clip = FrameRect {
        x: 12.0,
        y: 20.0,
        width: 180.0,
        height: 120.0,
    };
    let mut commands = Vec::new();

    assert!(push_dialog_commands(
        &mut commands,
        &node,
        &root,
        &clip,
        0,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn narrow_dialog_keeps_chrome_but_omits_text_that_cannot_fit_inside_the_surface() {
    let node = TemplatePaneNodeData {
        role: "Dialog".to_string(),
        popup_open: true,
        text: "Scene Settings".to_string(),
        ..TemplatePaneNodeData::default()
    };
    let root = FrameRect {
        x: 20.0,
        y: 20.0,
        width: 20.0,
        height: 20.0,
    };
    let mut commands = Vec::new();

    assert!(push_dialog_commands(
        &mut commands,
        &node,
        &root,
        &root,
        0,
        1.0,
    ));

    assert!(commands
        .iter()
        .all(|command| !matches!(command.kind, HostPaintCommandKind::Text)));
}
