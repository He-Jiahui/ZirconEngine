use super::centered_line_y;

#[test]
fn centered_line_y_does_not_move_a_taller_line_above_its_frame() {
    assert_eq!(centered_line_y(10.0, 20.0, 12.0), 14.0);
    assert_eq!(centered_line_y(10.0, 8.0, 12.0), 10.0);
}
