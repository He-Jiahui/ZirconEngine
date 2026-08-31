mod segments;

pub(super) fn validate_runtime_plugin_package_bevy_reference_path(
    capability: &str,
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    if !reference.starts_with("dev/bevy/") {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must stay under `dev/bevy`"
        ));
    }
    if contains_non_repository_path_separator(reference) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must be a repository-relative forward-slash path"
        ));
    }
    segments::validate_runtime_plugin_package_bevy_reference_path_segments(
        capability,
        reference,
        diagnostics,
    );
}

fn contains_non_repository_path_separator(reference: &str) -> bool {
    reference.bytes().any(|byte| matches!(byte, b'\\' | b':'))
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::contains_non_repository_path_separator;

    const SAMPLE_BYTES: usize = 4_096;
    const CHECKS_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_gd_runtime486_reference_separator_scan_preserves_rejection_set() {
        assert!(contains_non_repository_path_separator("dev\\bevy/render"));
        assert!(contains_non_repository_path_separator("C:/dev/bevy/render"));
        assert!(contains_non_repository_path_separator(
            "dev/bevy/render:note"
        ));
        assert!(!contains_non_repository_path_separator("dev/bevy/render"));
        assert!(!contains_non_repository_path_separator("dev/bevy/渲染"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_gd_runtime486_reference_separator_scan_benchmark() {
        let mut input = "a".repeat(SAMPLE_BYTES);
        input.push(':');
        for _ in 0..4 {
            black_box(measure_checks(&input, false));
            black_box(measure_checks(&input, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(&input, false));
                optimized_samples.push(measure_checks(&input, true));
            } else {
                optimized_samples.push(measure_checks(&input, true));
                legacy_samples.push(measure_checks(&input, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME486_REFERENCE_SEPARATOR_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_full_scans_per_check=2 optimized_full_scans_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
            input.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 80 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let rejected = if optimized {
                contains_non_repository_path_separator(black_box(input))
            } else {
                let input = black_box(input);
                input.contains('\\') || input.contains(':')
            };
            black_box(rejected);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
