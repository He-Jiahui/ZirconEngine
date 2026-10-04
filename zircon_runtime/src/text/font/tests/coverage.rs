use super::FontCoverage;

#[test]
fn coverage_contains_uses_ordered_ranges_without_gap_false_positives() {
    let coverage = FontCoverage::Known(vec![(0x0020, 0x007E), (0x0400, 0x04FF)]);

    assert!(coverage.contains('A'));
    assert!(coverage.contains('\u{416}'));
    assert!(!coverage.contains('\u{4e2d}'));
}
