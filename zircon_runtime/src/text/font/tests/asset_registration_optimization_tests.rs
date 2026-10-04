use super::normalized_family_matches;

#[test]
fn runtime80_batch_primary_family_match_trims_and_folds_ascii_case() {
    assert!(normalized_family_matches(
        "  Runtime Sans Regular  ",
        "runtime sans regular"
    ));
}

#[test]
fn runtime80_batch_primary_family_match_keeps_non_ascii_case_strict() {
    assert!(normalized_family_matches("Familie Ö", "familie Ö"));
    assert!(!normalized_family_matches("Familie Ö", "familie ö"));
}
