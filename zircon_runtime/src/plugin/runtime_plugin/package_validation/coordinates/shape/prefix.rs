use super::super::super::is_lowercase_runtime_plugin_token;

pub(in crate::plugin::runtime_plugin::package_validation::coordinates) fn validate_runtime_plugin_package_coordinate_prefix(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.is_empty()
        || has_outer_whitespace(value)
        || value
            .split('.')
            .any(|segment| !is_lowercase_runtime_plugin_token(segment))
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must contain only non-empty lowercase coordinate segments"
        ));
    }
}

fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{has_outer_whitespace, validate_runtime_plugin_package_coordinate_prefix};

    const COORDINATE_BYTES: usize = 2_048 * 1_024;
    const CHECKS_PER_SAMPLE: usize = 1_024;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fy_runtime481_coordinate_prefix_edge_check_preserves_rejection() {
        assert!(has_outer_whitespace(" com.zircon.runtime "));
        assert!(!has_outer_whitespace("com.zircon.runtime"));

        let mut diagnostics = Vec::new();
        validate_runtime_plugin_package_coordinate_prefix(
            "package coordinate",
            "com.zircon.runtime ",
            &mut diagnostics,
        );
        assert_eq!(diagnostics.len(), 1);
        diagnostics.clear();
        validate_runtime_plugin_package_coordinate_prefix(
            "package coordinate",
            "com.zircon.runtime",
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fy_runtime481_coordinate_prefix_edge_check_benchmark() {
        let mut coordinate = String::with_capacity(COORDINATE_BYTES + 16);
        coordinate.push_str("com.zircon.runtime");
        coordinate.extend(std::iter::repeat_n(' ', COORDINATE_BYTES));
        for _ in 0..4 {
            black_box(measure_checks(&coordinate, false));
            black_box(measure_checks(&coordinate, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(&coordinate, false));
                optimized_samples.push(measure_checks(&coordinate, true));
            } else {
                optimized_samples.push(measure_checks(&coordinate, true));
                legacy_samples.push(measure_checks(&coordinate, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME481_COORDINATE_PREFIX_EDGE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} coordinate_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_trim_scans_per_check=2 optimized_edge_scans_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=75",
            coordinate.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 25 / 100);
    }

    fn measure_checks(value: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let has_outer = if optimized {
                has_outer_whitespace(black_box(value))
            } else {
                let input = black_box(value);
                input.trim().is_empty() || input.trim() != input
            };
            black_box(has_outer);
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
