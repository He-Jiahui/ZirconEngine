use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::METRICS;
use super::super::super::template_segmented_control_geometry::{
    segment_rect, segmented_body_rect, workbench_segmented_control_metrics_from_host,
};
use super::support::{frame_rect, labeled_segmented_node};

#[test]
fn segment_rects_split_available_width_evenly() {
    let rect = FrameRect {
        x: 6.0,
        y: 4.0,
        width: 150.0,
        height: 30.0,
    };

    assert_eq!(segment_rect(&rect, 0, 3).x, 6.0);
    assert_eq!(segment_rect(&rect, 1, 3).x, 56.0);
    assert_eq!(segment_rect(&rect, 2, 3).width, 50.0);
}

#[test]
fn segmented_control_offsets_group_label_body() {
    let node = labeled_segmented_node();
    let body = segmented_body_rect(&node, &frame_rect(&node.frame));

    assert_eq!(body.x, 18.0);
    assert_eq!(body.y, 22.0);
    assert_eq!(body.height, 30.0);
}

#[test]
fn segmented_control_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.radius_control = 3.0;
    host.border_width = 1.5;
    host.font_body = 11.0;
    host.line_height_ratio = 1.25;
    host.button_pad_x = 13.0;
    host.input_pad[0] = 9.0;
    host.segment_text_inset_y = 5.0;
    host.segment_selected_inset = 3.0;
    host.tab_underline_height = 3.0;
    host.gap_s = 6.0;

    let metrics = workbench_segmented_control_metrics_from_host(host);

    assert_close(metrics.segment_font_size, 11.0);
    assert_close(metrics.segment_line_height, 13.75);
    assert_close(metrics.segment_text_inset_x, 9.0);
    assert_close(metrics.segment_text_inset_y, 5.0);
    assert_close(metrics.segment_radius, 3.0);
    assert_close(metrics.segment_group_label_font_size, 11.0);
    assert_close(metrics.segment_group_label_line_height, 13.75);
    assert_close(metrics.segment_group_label_height, 16.75);
    assert_close(metrics.segment_group_label_gap, 6.0);
    assert_close(metrics.segment_selected_inset, 3.0);
    assert_close(metrics.segment_divider_width, 1.5);
    assert_close(metrics.segment_divider_inset_y, 6.0);
    assert_close(metrics.tab_font_size, 11.0);
    assert_close(metrics.tab_line_height, 13.75);
    assert_close(metrics.tab_underline_height, 3.0);
    assert_close(metrics.tab_text_inset_x, 13.0);
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {actual} to be close to {expected}"
    );
}
