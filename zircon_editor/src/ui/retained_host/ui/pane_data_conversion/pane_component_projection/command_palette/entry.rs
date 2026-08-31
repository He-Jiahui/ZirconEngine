#[derive(Clone, Debug)]
pub(super) struct CommandProjectionEntry {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) description: String,
    pub(super) disabled: bool,
    pub(super) filter_matched: bool,
}

impl CommandProjectionEntry {
    pub(super) fn new(id: String) -> Self {
        Self {
            label: id.clone(),
            id,
            description: String::new(),
            disabled: false,
            filter_matched: false,
        }
    }

    pub(super) fn with_filter_matched(mut self) -> Self {
        self.filter_matched = true;
        self
    }

    pub(super) fn matches_query(&self, query: Option<&str>) -> bool {
        let Some(query) = query.map(str::trim).filter(|query| !query.is_empty()) else {
            return false;
        };
        if contains_ascii_case_insensitive(&self.id, query) {
            return true;
        }
        if self.label.len() == self.id.len()
            && (query.len() > self.id.len() || self.label == self.id)
        {
            return false;
        }
        contains_ascii_case_insensitive(&self.label, query)
    }
}

fn contains_ascii_case_insensitive(value: &str, expected: &str) -> bool {
    expected.is_empty()
        || value
            .as_bytes()
            .windows(expected.len())
            .any(|window| window.eq_ignore_ascii_case(expected.as_bytes()))
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    fn legacy_matches_query(entry: &CommandProjectionEntry, query: Option<&str>) -> bool {
        let Some(query) = query.map(str::trim).filter(|query| !query.is_empty()) else {
            return false;
        };
        contains_ascii_case_insensitive(&entry.id, query)
            || contains_ascii_case_insensitive(&entry.label, query)
    }

    #[test]
    fn optimization_batch_er_command_query_skips_duplicate_label_scan() {
        let default_entry = CommandProjectionEntry::new("Editor.Command.Open".to_string());
        for query in [None, Some(""), Some("open"), Some("missing")] {
            assert_eq!(
                default_entry.matches_query(query),
                legacy_matches_query(&default_entry, query)
            );
        }

        let mut labeled_entry = default_entry.clone();
        labeled_entry.label = "Open Project".to_string();
        assert_eq!(
            labeled_entry.matches_query(Some("project")),
            legacy_matches_query(&labeled_entry, Some("project"))
        );

        let source = include_str!("entry.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("command projection production implementation");
        assert!(production.contains("self.label == self.id"));
    }

    #[test]
    #[ignore = "release-only duplicate command label scan benchmark"]
    fn optimization_batch_er_duplicate_command_label_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const CHECKS_PER_SAMPLE: usize = 32_768;
        const TEXT_BYTES: usize = 512;

        fn measure_legacy(entry: &CommandProjectionEntry, query: &str) -> u128 {
            let started = Instant::now();
            let mut matched = 0_usize;
            for _ in 0..CHECKS_PER_SAMPLE {
                matched += usize::from(legacy_matches_query(
                    black_box(entry),
                    Some(black_box(query)),
                ));
            }
            black_box(matched);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(entry: &CommandProjectionEntry, query: &str) -> u128 {
            let started = Instant::now();
            let mut matched = 0_usize;
            for _ in 0..CHECKS_PER_SAMPLE {
                matched += usize::from(black_box(entry).matches_query(Some(black_box(query))));
            }
            black_box(matched);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let entry = CommandProjectionEntry::new("a".repeat(TEXT_BYTES));
        let query = format!("{}b", "a".repeat(31));
        assert!(!entry.matches_query(Some(&query)));
        for _ in 0..4 {
            black_box(measure_legacy(&entry, &query));
            black_box(measure_optimized(&entry, &query));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&entry, &query));
                optimized_samples.push(measure_optimized(&entry, &query));
            } else {
                optimized_samples.push(measure_optimized(&entry, &query));
                legacy_samples.push(measure_legacy(&entry, &query));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR380_DUPLICATE_COMMAND_LABEL_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             checks_per_sample={CHECKS_PER_SAMPLE} text_bytes={TEXT_BYTES} \
             pair_order=alternating_legacy_even legacy_text_scans_per_check=2 \
             optimized_text_scans_per_check=1 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(65),
            "duplicate command label scans must reduce P95 by at least 35%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
