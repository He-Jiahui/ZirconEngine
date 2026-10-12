use super::super::metrics::notification_center_metrics;
use super::*;

#[test]
fn rows_and_content_slots_stay_inside_a_narrow_short_panel() {
    let panel = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 18.0,
        height: 48.0,
    };
    let metrics = notification_center_metrics();

    for index in 0..=2 {
        let row = row_rect(&panel, index, &metrics);
        let width = row_text_width(&row, &metrics);

        assert_contained(row.clone(), &panel);
        assert_contained(mark_rect(&row, &metrics), &row);
        assert_contained(title_rect(&row, width, &metrics), &row);
        assert_contained(message_rect(&row, width, &metrics), &row);
    }
}

fn assert_contained(rect: FrameRect, parent: &FrameRect) {
    let epsilon = 0.000_1;
    assert!(rect.x >= parent.x - epsilon);
    assert!(rect.y >= parent.y - epsilon);
    assert!(rect.x + rect.width <= parent.x + parent.width + epsilon);
    assert!(rect.y + rect.height <= parent.y + parent.height + epsilon);
}
