use super::*;

#[test]
fn physical_pointer_coordinates_cross_the_inverse_scale_boundary_once() {
    assert_eq!(logical_axis_from_physical(148.0, 100.0, 2.0), 24.0);
    assert_eq!(logical_axis_from_physical(148.0, 100.0, 0.0), 48.0);
}

#[test]
fn logical_frames_cross_the_physical_boundary_once() {
    assert_eq!(
        scale_frame(UiFrame::new(8.0, 12.0, 80.0, 24.0), 2.0),
        UiFrame::new(16.0, 24.0, 160.0, 48.0)
    );
}
