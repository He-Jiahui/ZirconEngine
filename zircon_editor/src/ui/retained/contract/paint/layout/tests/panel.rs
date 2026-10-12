use super::super::metrics::notification_center_metrics;
use super::*;

#[test]
fn header_and_empty_slots_stay_inside_a_tiny_notification_panel() {
    let panel = FrameRect {
        x: 11.0,
        y: 22.0,
        width: 18.0,
        height: 12.0,
    };
    let metrics = notification_center_metrics();

    assert_contained(header_rect(&panel, &metrics), &panel);
    assert_contained(empty_text_rect(&panel, &metrics), &panel);
}

fn assert_contained(rect: FrameRect, parent: &FrameRect) {
    let epsilon = 0.000_1;
    assert!(rect.x >= parent.x - epsilon);
    assert!(rect.y >= parent.y - epsilon);
    assert!(rect.x + rect.width <= parent.x + parent.width + epsilon);
    assert!(rect.y + rect.height <= parent.y + parent.height + epsilon);
}
