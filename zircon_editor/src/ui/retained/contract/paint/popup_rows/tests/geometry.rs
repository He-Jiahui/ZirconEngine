use super::*;

#[test]
fn popup_row_frame_rejects_collapsed_non_finite_and_outside_geometry() {
    let outer = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 24.0,
        height: 18.0,
    };

    assert!(frame_is_within(
        &outer,
        &FrameRect {
            x: 5.0,
            y: 9.0,
            width: 12.0,
            height: 8.0,
        }
    ));
    assert!(!has_paintable_popup_row_extent(&FrameRect {
        width: 0.0,
        ..outer.clone()
    }));
    assert!(!has_paintable_popup_row_extent(&FrameRect {
        x: f32::NAN,
        ..outer.clone()
    }));
    assert!(!frame_is_within(
        &outer,
        &FrameRect {
            x: 20.0,
            y: 9.0,
            width: 12.0,
            height: 8.0,
        }
    ));
}
