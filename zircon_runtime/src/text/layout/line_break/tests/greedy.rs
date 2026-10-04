use super::should_wrap_before_accumulated;

#[test]
fn accumulated_wrap_uses_existing_and_next_advances_without_text_candidates() {
    assert!(!should_wrap_before_accumulated(true, 8.0, 8.0, 10.0));
    assert!(!should_wrap_before_accumulated(false, 4.0, 6.0, 10.0));
    assert!(should_wrap_before_accumulated(false, 4.0, 6.1, 10.0));
    assert!(!should_wrap_before_accumulated(
        false,
        f32::MAX,
        f32::MAX,
        f32::INFINITY,
    ));
}
