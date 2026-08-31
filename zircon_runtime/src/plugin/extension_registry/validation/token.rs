pub(super) fn is_lowercase_plugin_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn is_lowercase_plugin_package_id(value: &str) -> bool {
    !value.is_empty()
        && !has_outer_whitespace(value)
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value.split('.').all(is_lowercase_plugin_package_segment)
}

fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

fn is_lowercase_plugin_package_segment(value: &str) -> bool {
    is_lowercase_plugin_token(value) && !value.ends_with('_') && !value.contains("__")
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{has_outer_whitespace, is_lowercase_plugin_package_id};

    const SAMPLE_BYTES: usize = 2_048 * 1_024;
    const CHECKS_PER_SAMPLE: usize = 1_024;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_ga_runtime483_package_token_edge_check_preserves_trim_semantics() {
        for value in [
            " com.zircon.runtime",
            "com.zircon.runtime ",
            "\u{2003}com.zircon.runtime",
        ] {
            assert!(has_outer_whitespace(value));
            assert!(!is_lowercase_plugin_package_id(value));
        }
        assert!(is_lowercase_plugin_package_id("com.zircon.runtime"));
        assert!(!is_lowercase_plugin_package_id("com.zircon..runtime"));
        assert!(!is_lowercase_plugin_package_id("Com.zircon.runtime"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ga_runtime483_package_token_edge_check_benchmark() {
        let mut input = String::with_capacity(SAMPLE_BYTES + 19);
        input.extend(std::iter::repeat_n(' ', SAMPLE_BYTES));
        input.push_str("com.zircon.runtime");
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
            "RUNTIME483_PACKAGE_TOKEN_EDGE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} sample_bytes={SAMPLE_BYTES} checks_per_sample={CHECKS_PER_SAMPLE} legacy_trim_scans_per_check=2 optimized_edge_scans_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=75",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 25 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let valid = if optimized {
                let value = black_box(input);
                !value.is_empty() && !has_outer_whitespace(value)
            } else {
                let value = black_box(input);
                !value.trim().is_empty() && value.trim() == value
            };
            black_box(valid);
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
