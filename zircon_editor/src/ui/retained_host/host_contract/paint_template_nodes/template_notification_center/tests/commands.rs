use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::TemplatePaneOptionData;

use super::super::instrumentation::{reset, snapshot, NotificationPaintCounters};

#[test]
fn visible_rows_include_one_overscan_row_on_each_side() {
    let metrics = notification_center_metrics();
    let stride = metrics.row_height + metrics.row_gap;
    let panel = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 320.0,
        height: metrics.row_top + stride * 40.0,
    };
    let clip = FrameRect {
        x: panel.x,
        y: panel.y + metrics.row_top + stride * 10.25,
        width: panel.width,
        height: stride * 2.5,
    };

    assert_eq!(
        notification_center_visible_rows(&panel, &clip, 40, &metrics),
        9..14
    );
}

#[test]
fn visible_rows_are_empty_for_disjoint_or_non_finite_clip() {
    let metrics = notification_center_metrics();
    let panel = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 320.0,
        height: 240.0,
    };
    let disjoint = FrameRect {
        x: 421.0,
        y: panel.y,
        width: 20.0,
        height: panel.height,
    };
    let non_finite = FrameRect {
        x: f32::NAN,
        ..disjoint.clone()
    };

    assert_eq!(
        notification_center_visible_rows(&panel, &disjoint, 64, &metrics),
        0..0
    );
    assert_eq!(
        notification_center_visible_rows(&panel, &non_finite, 64, &metrics),
        0..0
    );
}

#[test]
fn painter_source_has_no_full_row_loop_or_cloning_row_access() {
    let source = include_str!("../commands.rs");
    let full_loop = ["for row in ", "0..row_count"].concat();
    let cloning_access = ["structured_options", ".row_data(row)"].concat();
    let borrowed_access = ["structured_options", ".get(row)"].concat();

    assert!(!source.contains(&full_loop));
    assert!(!source.contains(&cloning_access));
    assert!(source.contains(&borrowed_access));
}

#[test]
fn closed_center_performs_no_palette_metrics_or_row_work() {
    let metrics = notification_center_metrics();
    let rect = notification_test_rect(&metrics);
    let clip = notification_row_clip(&rect, &metrics, 10);
    let node = notification_test_node(false);
    let mut commands = Vec::new();

    reset();
    assert!(push_notification_center_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    assert_eq!(snapshot(), NotificationPaintCounters::default());
    assert!(commands.is_empty());
}

#[test]
fn open_center_visits_only_visible_and_overscan_rows_and_copies_visible_text() {
    let metrics = notification_center_metrics();
    let rect = notification_test_rect(&metrics);
    let clip = notification_row_clip(&rect, &metrics, 10);
    let node = notification_test_node(true);
    let mut commands = Vec::new();

    reset();
    assert!(push_notification_center_commands(
        &mut commands,
        &node,
        &rect,
        &clip,
        0,
        1.0,
    ));

    let counters = snapshot();
    assert_eq!(counters.palette_resolutions, 1);
    assert_eq!(counters.metrics_resolutions, 1);
    assert_eq!(counters.row_count_reads, 1);
    assert_eq!(counters.row_visits, 3);
    assert_eq!(counters.title_text_copies, 1);
    assert_eq!(counters.message_text_copies, 1);
}

#[test]
fn node_panel_radius_overrides_the_host_fallback_without_flattening_row_radius() {
    let mut node = notification_test_node(true);
    node.corner_radius = 14.0;

    let metrics = notification_center_metrics_for_node(&node);

    assert_eq!(metrics.panel_radius, 14.0);
    assert_eq!(metrics.row_radius, 6.0);
}

fn notification_test_node(popup_open: bool) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "notification-center".into(),
        popup_open,
        structured_options: model_rc(
            (0..64)
                .map(|index| TemplatePaneOptionData {
                    id: format!("notification-{index}").into(),
                    label: format!("Notification {index}").into(),
                    description: format!("Message {index}").into(),
                    ..TemplatePaneOptionData::default()
                })
                .collect(),
        ),
        ..TemplatePaneNodeData::default()
    }
}

fn notification_test_rect(metrics: &super::super::layout::NotificationCenterMetrics) -> FrameRect {
    let stride = metrics.row_height + metrics.row_gap;
    FrameRect {
        x: 100.0,
        y: 50.0,
        width: 320.0,
        height: metrics.row_top + stride * 64.0,
    }
}

fn notification_row_clip(
    rect: &FrameRect,
    metrics: &super::super::layout::NotificationCenterMetrics,
    row: usize,
) -> FrameRect {
    let stride = metrics.row_height + metrics.row_gap;
    FrameRect {
        x: rect.x,
        y: rect.y + metrics.row_top + stride * row as f32 + 1.0,
        width: rect.width,
        height: metrics.row_height - 2.0,
    }
}
