use super::*;

#[test]
fn axis_label_text_rect_stays_inside_an_empty_or_short_slot() {
    let rect = FrameRect {
        x: 4.0,
        y: 10.0,
        width: 0.0,
        height: 24.0,
    };
    let metrics = AxisLabelMetrics {
        font_size: 11.0,
        line_height: 14.0,
        link_lobe_width: 6.0,
        link_lobe_height: 7.0,
        link_lobe_radius: 3.0,
        link_overlap: 2.0,
        link_connector_width: 1.0,
    };

    assert_eq!(
        axis_label_text_rect(&rect, &metrics),
        FrameRect {
            x: 4.0,
            y: 15.0,
            width: 0.0,
            height: 14.0,
        }
    );

    let short = FrameRect {
        x: 4.0,
        y: 10.0,
        width: 12.0,
        height: 6.0,
    };
    assert_eq!(
        axis_label_text_rect(&short, &metrics),
        FrameRect {
            x: 4.0,
            y: 10.0,
            width: 12.0,
            height: 6.0,
        }
    );
}

#[test]
fn axis_label_text_rect_collapses_invalid_frame_values() {
    let metrics = AxisLabelMetrics {
        font_size: 11.0,
        line_height: 14.0,
        link_lobe_width: 6.0,
        link_lobe_height: 7.0,
        link_lobe_radius: 3.0,
        link_overlap: 2.0,
        link_connector_width: 1.0,
    };
    let frame = axis_label_text_rect(
        &FrameRect {
            x: f32::NAN,
            y: f32::INFINITY,
            width: f32::NEG_INFINITY,
            height: f32::NAN,
        },
        &metrics,
    );

    assert_eq!(frame.x, 0.0);
    assert_eq!(frame.y, 0.0);
    assert_eq!(frame.width, 0.0);
    assert_eq!(frame.height, 0.0);
}
