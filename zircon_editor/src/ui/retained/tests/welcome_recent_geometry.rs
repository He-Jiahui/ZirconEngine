use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

const EPSILON: f32 = 0.01;

#[test]
fn welcome_recent_viewport_prefers_the_projected_list_frame() {
    let projected = FrameRect {
        x: 37.0,
        y: 91.0,
        width: 184.0,
        height: 308.0,
    };
    let layout = WelcomePaneLayoutData {
        recent_list_panel: Some(projected.clone()),
        ..WelcomePaneLayoutData::default()
    };

    assert_eq!(
        welcome_recent_viewport_for_layout(&layout, UiSize::new(640.0, 520.0)),
        UiFrame::new(projected.x, projected.y, projected.width, projected.height)
    );
    assert_ne!(
        welcome_recent_viewport_for_layout(&layout, UiSize::new(640.0, 520.0)),
        welcome_recent_viewport(UiSize::new(640.0, 520.0))
    );
}

#[test]
fn welcome_recent_viewport_does_not_resurrect_a_collapsed_authoritative_panel() {
    let current = WelcomePaneLayoutData {
        has_nodes: true,
        ..WelcomePaneLayoutData::default()
    };
    let legacy = WelcomePaneLayoutData::default();

    assert_eq!(
        welcome_recent_viewport_for_layout(&current, UiSize::new(640.0, 520.0)),
        UiFrame::new(0.0, 0.0, 0.0, 0.0)
    );
    assert!(welcome_recent_viewport_for_layout(&legacy, UiSize::new(640.0, 520.0)).width > 0.0);
}

#[test]
fn welcome_recent_geometry_keeps_compact_rows_and_actions_inside_responsive_columns() {
    let metrics = welcome_recent_layout_metrics_from_host(METRICS);
    assert_close(metrics.outer_inset, 18.0);
    assert_close(metrics.header_height, 46.0);
    assert_close(metrics.row_height, 54.0);
    assert_close(metrics.row_action_height, 24.0);
    assert_close(metrics.open_action_width, 52.0);
    assert_close(metrics.safe_action_width, 24.0);
    assert_close(metrics.recover_action_width, 24.0);
    for (pane_width, expected_recent_width) in [(560.0, 244.0), (640.0, 320.0), (900.0, 320.0)] {
        let viewport =
            welcome_recent_viewport_with_metrics(UiSize::new(pane_width, 520.0), metrics);
        assert_close(viewport.width, expected_recent_width);

        let first = welcome_recent_row_geometry_with_metrics(viewport, 0, 0.0, metrics);
        let second = welcome_recent_row_geometry_with_metrics(viewport, 1, 0.0, metrics);
        assert_close(first.row.height, 54.0);
        assert_close(second.row.y - first.row.y, 62.0);
        assert!(first.text.x >= first.row.x);
        assert!(first.text.right() <= first.open.x);
        assert!(first.open.right() <= first.safe.x);
        assert!(first.safe.right() <= first.recover.x);
        assert!(first.recover.right() <= first.remove.x);
        assert!(first.remove.right() <= first.row.right());
        for action in [first.open, first.safe, first.recover, first.remove] {
            assert!(action.y >= first.row.y);
            assert!(action.bottom() <= first.row.bottom());
        }
    }
}

#[test]
fn welcome_recent_geometry_reserves_a_recovery_action_between_open_and_remove() {
    let metrics = welcome_recent_layout_metrics_from_host(METRICS);
    let viewport = welcome_recent_viewport_with_metrics(UiSize::new(640.0, 520.0), metrics);
    let row = welcome_recent_row_geometry_with_metrics(viewport, 0, 0.0, metrics);

    assert!(row.text.right() <= row.open.x);
    assert!(row.open.right() <= row.safe.x);
    assert!(row.safe.right() <= row.recover.x);
    assert!(row.recover.right() <= row.remove.x);
    assert!(row.remove.right() <= row.row.right());
    assert_close(row.recover.width, metrics.row_action_height);
}

#[test]
fn welcome_recent_geometry_derives_content_and_visible_rows_from_one_metric_owner() {
    let metrics = welcome_recent_layout_metrics_from_host(METRICS);
    assert_close(welcome_recent_content_height_with_metrics(0, metrics), 0.0);
    assert_close(welcome_recent_content_height_with_metrics(1, metrics), 70.0);
    assert_close(
        welcome_recent_content_height_with_metrics(3, metrics),
        194.0,
    );
    assert_eq!(
        welcome_recent_visible_row_count_with_metrics(0.0, 8, metrics),
        0
    );
    assert_eq!(
        welcome_recent_visible_row_count_with_metrics(70.0, 8, metrics),
        1
    );
    assert_eq!(
        welcome_recent_visible_row_count_with_metrics(132.0, 8, metrics),
        2
    );
    assert_eq!(
        welcome_recent_visible_row_count_with_metrics(520.0, 2, metrics),
        2
    );
}

#[test]
fn welcome_recent_geometry_reflows_from_compact_host_density() {
    let mut host = METRICS;
    host.control_default_height = 28.0;
    host.control_large_height = 40.0;
    host.row_height = 24.0;
    host.border_width = 1.5;
    host.gap_s = 3.0;
    host.gap_m = 6.0;
    host.gap_l = 9.0;
    host.font_body = 10.0;
    host.line_height_ratio = 1.2;
    let metrics = welcome_recent_layout_metrics_from_host(host);
    let viewport = welcome_recent_viewport_with_metrics(UiSize::new(560.0, 420.0), metrics);
    let row = welcome_recent_row_geometry_with_metrics(viewport, 1, 2.0, metrics);

    assert_close(metrics.outer_inset, 15.0);
    assert_close(metrics.header_height, 37.0);
    assert_close(metrics.row_height, 43.0);
    assert_close(metrics.row_action_height, 21.0);
    assert_close(metrics.open_action_width, 43.0);
    assert_close(metrics.safe_action_width, 21.0);
    assert_close(metrics.recover_action_width, 21.0);
    assert_close(row.row.y, viewport.y + 6.0 + 43.0 + 6.0 - 2.0);
    assert_close(
        welcome_recent_content_height_with_metrics(2, metrics),
        104.0,
    );
    assert!(row.text.right() <= row.open.x);
    assert!(row.open.right() <= row.safe.x);
    assert!(row.safe.right() <= row.recover.x);
    assert!(row.recover.right() <= row.remove.x);
}

#[test]
fn welcome_recent_geometry_keeps_zero_width_actions_and_text_inside_narrow_rows() {
    let metrics = welcome_recent_layout_metrics_from_host(METRICS);
    let row = welcome_recent_row_geometry_with_metrics(
        UiFrame::new(0.0, 0.0, 12.0, 80.0),
        0,
        0.0,
        metrics,
    );

    assert_eq!(row.row.width, 0.0);
    assert_eq!(row.open.x, row.row.x);
    assert_eq!(row.safe.x, row.row.x);
    assert_eq!(row.recover.x, row.row.x);
    assert_eq!(row.remove.x, row.row.x);
    assert_eq!(row.text.x, row.row.x);
    assert_eq!(row.open.width, 0.0);
    assert_eq!(row.safe.width, 0.0);
    assert_eq!(row.recover.width, 0.0);
    assert_eq!(row.remove.width, 0.0);
    assert_eq!(row.text.width, 0.0);
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}
