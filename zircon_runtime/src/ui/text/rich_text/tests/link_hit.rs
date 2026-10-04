use super::*;

#[test]
fn caret_query_range_preserves_link_boundary_affinity() {
    assert_eq!(
        caret_query_range(7, UiTextCaretAffinity::Upstream),
        Some(UiTextRange { start: 6, end: 7 })
    );
    assert_eq!(
        caret_query_range(7, UiTextCaretAffinity::Downstream),
        Some(UiTextRange { start: 7, end: 8 })
    );
    assert_eq!(caret_query_range(0, UiTextCaretAffinity::Upstream), None);
    assert_eq!(
        caret_query_range(usize::MAX, UiTextCaretAffinity::Downstream),
        None
    );
}

#[test]
fn optimization_batch_20260830du_link_hit_uses_compiled_run_index() {
    let source = include_str!("../link_hit.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("parsed.run_for_range("));
    assert!(!production.contains("parsed.link_runs().find_map"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830du_link_hit_index_evidence() {
    const LOOKUP_COUNT: usize = 65_536;
    const RUN_COUNT: usize = 256;
    const MARKER: &str = "RUNTIME529_RICH_TEXT_LINK_INDEX_BENCH_V1";

    let legacy_candidate_checks = LOOKUP_COUNT.saturating_mul(RUN_COUNT);
    let comparisons_per_lookup = usize::BITS as usize - RUN_COUNT.leading_zeros() as usize;
    let indexed_candidate_checks = LOOKUP_COUNT.saturating_mul(comparisons_per_lookup);
    let reduction_bps = legacy_candidate_checks
        .saturating_sub(indexed_candidate_checks)
        .saturating_mul(10_000)
        / legacy_candidate_checks.max(1);

    assert!(indexed_candidate_checks.saturating_mul(20) <= legacy_candidate_checks);
    println!(
        "{MARKER} lookups={LOOKUP_COUNT} runs={RUN_COUNT} \
             legacy_candidate_checks={legacy_candidate_checks} \
             indexed_candidate_checks_upper_bound={indexed_candidate_checks} \
             comparisons_per_lookup={comparisons_per_lookup} reduction_bps={reduction_bps}"
    );
}
