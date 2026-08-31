use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginFeatureSelection, ProjectPluginSelection};

pub(super) fn target_consumes_selection(
    selection: &ProjectPluginSelection,
    target: RuntimeTargetMode,
) -> bool {
    selection.enabled && selection.supports_target(target)
}

pub(super) fn target_consumes_feature(
    feature: &ProjectPluginFeatureSelection,
    target: RuntimeTargetMode,
) -> bool {
    feature.enabled && feature.supports_target(target)
}

pub(super) fn is_lowercase_project_plugin_package_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn is_lowercase_project_feature_namespace(value: &str) -> bool {
    let mut saw_separator = false;
    let mut segment_is_non_empty = false;
    for byte in value.bytes() {
        if byte == b'.' {
            if !segment_is_non_empty {
                return false;
            }
            saw_separator = true;
            segment_is_non_empty = false;
        } else if byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' {
            segment_is_non_empty = true;
        } else {
            return false;
        }
    }
    saw_separator && segment_is_non_empty
}

pub(super) fn is_lowercase_project_feature_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn is_lowercase_project_runtime_crate(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::is_lowercase_project_feature_namespace;

    #[test]
    fn feature_namespace_validation_does_not_collect_split_segments() {
        let source = include_str!("tokens.rs");
        let allocating_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(
            !source.contains(&allocating_shape),
            "feature namespace validation should stream split segments without a temporary Vec"
        );
    }

    #[test]
    fn optimization_batch_ge_runtime487_feature_namespace_single_scan_preserves_segments() {
        for value in ["rendering.ssao", "runtime.feature_2", "a.b.c"] {
            assert!(is_lowercase_project_feature_namespace(value), "{value}");
        }
        for value in [
            "",
            "rendering",
            ".rendering",
            "rendering.",
            "rendering..ssao",
            "Rendering.ssao",
        ] {
            assert!(!is_lowercase_project_feature_namespace(value), "{value}");
        }
    }

    const SAMPLE_BYTES: usize = 2_048 * 1_024;
    const CHECKS_PER_SAMPLE: usize = 256;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ge_runtime487_feature_namespace_single_scan_benchmark() {
        let mut input = "a".repeat(SAMPLE_BYTES);
        input.push_str(".feature");
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
            "RUNTIME487_FEATURE_NAMESPACE_SINGLE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_full_passes_per_check=3 optimized_full_passes_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
            input.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 80 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let valid = if optimized {
                is_lowercase_project_feature_namespace(black_box(input))
            } else {
                legacy_is_lowercase_project_feature_namespace(black_box(input))
            };
            black_box(valid);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_is_lowercase_project_feature_namespace(value: &str) -> bool {
        value.contains('.')
            && value.split('.').all(|segment| {
                !segment.is_empty()
                    && segment.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            })
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
