use super::{next_uniform_popup_row_at_boundary, uniform_popup_row_at_y};

#[test]
fn uniform_popup_row_lookup_is_constant_time_and_preserves_inclusive_boundaries() {
    assert_eq!(uniform_popup_row_at_y(10.0, 10.0, 24.0, 10_000), Some(0));
    assert_eq!(uniform_popup_row_at_y(34.0, 10.0, 24.0, 10_000), Some(0));
    assert_eq!(
        next_uniform_popup_row_at_boundary(34.0, 10.0, 24.0, 0, 10_000),
        Some(1)
    );
    assert_eq!(uniform_popup_row_at_y(58.1, 10.0, 24.0, 10_000), Some(2));
    assert_eq!(uniform_popup_row_at_y(9.9, 10.0, 24.0, 10_000), None);
}
