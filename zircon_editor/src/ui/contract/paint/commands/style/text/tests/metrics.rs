use super::*;

#[test]
fn measured_width_is_non_negative() {
    assert_eq!(measured_text_width(-4.0), 0.0);
    assert_eq!(measured_text_width(24.0), 24.0);
}

#[test]
fn text_alignment_positions_use_remaining_space() {
    assert_eq!(center_aligned_text_x(10.0, 100.0, 40.0), 40.0);
    assert_eq!(right_aligned_text_x(10.0, 100.0, 40.0), 70.0);
    assert_eq!(center_aligned_text_x(10.0, 20.0, 40.0), 10.0);
    assert_eq!(right_aligned_text_x(10.0, 20.0, 40.0), 10.0);
}
