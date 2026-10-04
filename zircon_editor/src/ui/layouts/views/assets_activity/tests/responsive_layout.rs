use super::fit_horizontal_pair;

#[test]
fn horizontal_pair_never_exceeds_an_ultra_narrow_budget() {
    let (primary, secondary, gap) = fit_horizontal_pair(16.0, 28.0, 64.0, 4.0);

    assert_eq!(primary, 16.0);
    assert_eq!(secondary, 0.0);
    assert_eq!(gap, 0.0);
    assert!(primary + gap + secondary <= 16.0);
}

#[test]
fn horizontal_pair_keeps_preferred_controls_and_standard_gap_when_space_allows() {
    let (primary, secondary, gap) = fit_horizontal_pair(120.0, 32.0, 40.0, 4.0);

    assert_eq!((primary, secondary, gap), (32.0, 40.0, 4.0));
    assert!(primary + gap + secondary <= 120.0);
}
