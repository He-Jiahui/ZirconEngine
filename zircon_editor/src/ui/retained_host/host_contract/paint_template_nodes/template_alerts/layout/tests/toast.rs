use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn toast_action_uses_metric_derived_content_budget() {
    let metrics = super::super::metrics::toast_metrics_from_host(METRICS);
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: metrics.action_minimum_width - 0.1,
        height: METRICS.row_height,
    };
    assert!(!toast_has_action(&rect, metrics));

    let mut wide_rect = rect;
    wide_rect.width = metrics.action_minimum_width;
    assert!(toast_has_action(&wide_rect, metrics));
}

#[test]
fn compact_toast_chrome_stays_inside_its_frame() {
    let metrics = super::super::metrics::toast_metrics_from_host(METRICS);
    let rect = FrameRect {
        x: 5.0,
        y: 7.0,
        width: 20.0,
        height: 10.0,
    };

    let icon = toast_icon_rect(&rect, metrics.icon_size, metrics);
    assert_eq!(icon.x, 17.0);
    assert_eq!(icon.y, 8.0);
    assert_eq!(icon.width, 8.0);
    assert_eq!(icon.height, 8.0);

    let close = toast_close_rect(&rect, metrics);
    assert_eq!(close.x, 5.0);
    assert_eq!(close.y, 7.0);
    assert_eq!(close.width, 10.0);
    assert_eq!(close.height, 10.0);
    assert!(!toast_has_action(&rect, metrics));
}
