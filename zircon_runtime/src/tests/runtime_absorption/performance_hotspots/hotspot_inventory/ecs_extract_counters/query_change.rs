use super::super::sources::HotspotInventorySources;

pub(super) fn assert_query_and_change_evidence(sources: &HotspotInventorySources) {
    for required_query_anchor in [
        "const ENTITY_COUNT: usize = 128;",
        "const REPEATED_QUERY_RUNS: usize = 8;",
        "query_state_cache_stats_record_reuse_and_rebuild_counts",
        "query_state_reuses_archetype_matches_across_unchanged_frames",
        "assert_eq!(reused.cache_hits, REPEATED_QUERY_RUNS as u64)",
        "assert_eq!(reused.cache_misses, 1)",
        "assert_eq!(reused.cache_rebuilds, initial.cache_rebuilds)",
    ] {
        assert!(
            sources.query_tests.contains(required_query_anchor),
            "QueryState performance evidence should retain `{required_query_anchor}`"
        );
    }

    for required_change_anchor in [
        "change_detection_scan_stats_record_mark_checks_and_diagnostics",
        "change_detection_scan_skips_unmarked_archetypes",
        "assert_eq!(stats.scanned_marks, unmarked.len() as u64 * 2)",
        "assert_eq!(stats.added_matches, 0)",
        "assert_eq!(stats.changed_matches, 0)",
    ] {
        assert!(
            sources.change_tests.contains(required_change_anchor),
            "change-detection evidence should retain `{required_change_anchor}`"
        );
    }
}
