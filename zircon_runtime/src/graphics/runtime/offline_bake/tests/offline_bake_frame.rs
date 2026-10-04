use super::eligible_reflection_probe_count;

#[test]
fn eligible_probe_count_short_circuits_empty_and_clamps_budget() {
    assert_eq!(eligible_reflection_probe_count(0, 4, 1.0), 0);
    assert_eq!(eligible_reflection_probe_count(8, 0, 1.0), 0);
    assert_eq!(eligible_reflection_probe_count(8, 4, 0.0), 0);
    assert_eq!(eligible_reflection_probe_count(8, 4, f32::NAN), 0);
    assert_eq!(eligible_reflection_probe_count(8, 4, 1.0), 4);
    assert_eq!(eligible_reflection_probe_count(2, 4, 1.0), 2);
}
