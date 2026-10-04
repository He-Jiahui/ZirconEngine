use winit::dpi::{LogicalPosition, LogicalSize, Position, Size};
use zircon_runtime_interface::ZrRuntimeImeCursorAreaV1;

use super::{
    ime_candidate_rect, ime_candidate_rect_to_winit, ime_logical_cursor_area, ime_logical_position,
    ime_logical_size,
};

#[test]
fn ime_cursor_area_submits_window_logical_coordinates_without_dpi_scaling() {
    let area = ZrRuntimeImeCursorAreaV1::new(25.0, 68.5, 4.0, 37.0);

    match ime_logical_position(area) {
        Position::Logical(position) => {
            assert_eq!(position, LogicalPosition::new(25.0, 68.5));
        }
        position => panic!("expected logical IME position, got {position:?}"),
    }
    match ime_logical_size(area) {
        Size::Logical(size) => {
            assert_eq!(size, LogicalSize::new(4.0, 37.0));
        }
        size => panic!("expected logical IME size, got {size:?}"),
    }
}

#[test]
fn ime_cursor_area_rejects_non_finite_or_negative_size_geometry() {
    for area in [
        ZrRuntimeImeCursorAreaV1::new(f32::NAN, 68.5, 4.0, 37.0),
        ZrRuntimeImeCursorAreaV1::new(25.0, f32::INFINITY, 4.0, 37.0),
        ZrRuntimeImeCursorAreaV1::new(25.0, 68.5, -4.0, 37.0),
        ZrRuntimeImeCursorAreaV1::new(25.0, 68.5, 4.0, f32::NEG_INFINITY),
    ] {
        assert!(
            ime_logical_cursor_area(area).is_none(),
            "invalid ABI cursor geometry must not reach the window API: {area:?}"
        );
    }
}

#[test]
fn candidate_rect_rejects_native_coordinate_overflow_and_zero_height() {
    assert!(ime_candidate_rect(ZrRuntimeImeCursorAreaV1::new(f32::MAX, 1.0, 1.0, 10.0,)).is_none());
    assert!(ime_candidate_rect(ZrRuntimeImeCursorAreaV1::new(1.0, 1.0, 1.0, 0.0,)).is_none());
    assert!(ime_candidate_rect(ZrRuntimeImeCursorAreaV1::new(
        i32::MAX as f32,
        1.0,
        1.0,
        10.0,
    ))
    .is_none());
}

#[test]
fn candidate_rect_is_consumed_as_the_checked_winit_anchor_shape() {
    let candidate = ime_candidate_rect(ZrRuntimeImeCursorAreaV1::new(25.4, 68.5, 0.0, 37.1))
        .expect("representable candidate rect");
    assert_eq!(candidate.extent_width, 0);
    match ime_candidate_rect_to_winit(candidate).0 {
        Position::Logical(position) => assert_eq!(position, LogicalPosition::new(25.0, 69.0)),
        position => panic!("expected logical candidate position, got {position:?}"),
    }
}
