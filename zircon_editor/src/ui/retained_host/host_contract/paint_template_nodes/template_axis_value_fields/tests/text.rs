use super::*;

#[test]
fn text_slot_stays_inside_a_narrow_short_axis_field() {
    let field = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 4.0,
        height: 6.0,
    };
    let metrics = AxisValueFieldMetrics {
        max_height: 30.0,
        radius: 4.0,
        font_size: 13.33,
        line_height: 16.0,
        text_inset_x: 8.0,
    };

    assert_eq!(
        axis_field_text_rect(&field, metrics),
        FrameRect {
            x: 12.0,
            y: 20.0,
            width: 0.0,
            height: 6.0,
        }
    );
}

#[test]
fn text_slot_keeps_the_authored_regular_field_density() {
    let field = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 58.0,
        height: 24.0,
    };
    let metrics = AxisValueFieldMetrics {
        max_height: 30.0,
        radius: 4.0,
        font_size: 13.33,
        line_height: 16.0,
        text_inset_x: 8.0,
    };

    assert_eq!(
        axis_field_text_rect(&field, metrics),
        FrameRect {
            x: 16.0,
            y: 12.0,
            width: 42.0,
            height: 16.0,
        }
    );
}
