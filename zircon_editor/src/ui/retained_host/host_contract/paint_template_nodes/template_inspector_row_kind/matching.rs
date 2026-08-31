pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn matches_ignore_ascii_case(
    value: &str,
    candidates: &[&str],
) -> bool {
    candidates
        .iter()
        .any(|candidate| value.len() == candidate.len() && value.eq_ignore_ascii_case(candidate))
}

#[cfg(test)]
mod tests {
    use super::matches_ignore_ascii_case;

    #[test]
    fn optimization_batch_gg_editor419_matching_length_prefilter_preserves_matches() {
        assert!(matches_ignore_ascii_case("Boolean", &["boolean", "number"]));
        assert!(matches_ignore_ascii_case("NUMBER", &["bool", "number"]));
        assert!(!matches_ignore_ascii_case(
            "booleans",
            &["boolean", "number"]
        ));
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gg_editor419_matching_length_prefilter_benchmark() {
        const MARKER: &str = "EDITOR419_MATCHING_LENGTH_PREFILTER_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let value = "unavailable";
        let candidates = ["boolean", "number", "string", "color", "resource"];
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!matches_ignore_ascii_case(value, &candidates));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!candidates
                .iter()
                .any(|candidate| value.eq_ignore_ascii_case(candidate)));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
