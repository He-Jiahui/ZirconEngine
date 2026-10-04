use super::*;

#[test]
fn legend_steps_are_bounded_by_pixels_and_constant_budget() {
    assert_eq!(bounded_legend_steps(10, 100.0), 12);
    assert_eq!(bounded_legend_steps(usize::MAX, 10_000.0), 64);
    assert_eq!(bounded_legend_steps(usize::MAX, 3.0), 3);
}
