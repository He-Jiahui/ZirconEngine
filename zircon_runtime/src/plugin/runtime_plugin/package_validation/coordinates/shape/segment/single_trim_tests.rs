use std::hint::black_box;
use std::time::Instant;

use super::validate_runtime_plugin_package_coordinate_segment;
use crate::plugin::runtime_plugin::package_validation::is_lowercase_runtime_plugin_token;

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const SEGMENT_BYTES: usize = 4096;

fn legacy_validate(field_name: &str, value: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() || value.trim() != value || !is_lowercase_runtime_plugin_token(value)
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must be a non-empty lowercase coordinate segment"
        ));
    }
}

fn measure(value: &str, optimized: bool) -> u128 {
    let mut diagnostics = Vec::new();
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            validate_runtime_plugin_package_coordinate_segment(
                "package_id",
                black_box(value),
                &mut diagnostics,
            );
        } else {
            legacy_validate("package_id", black_box(value), &mut diagnostics);
        }
    }
    black_box(diagnostics.len());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn optimization_batch_20260829bb_runtime329_single_trim_coordinate_preserves_diagnostics() {
    for value in [
        "render_plugin",
        "",
        " ",
        " render_plugin",
        "render_plugin ",
        "RenderPlugin",
        "render-plugin",
    ] {
        let mut legacy = Vec::new();
        let mut optimized = Vec::new();
        legacy_validate("package_id", value, &mut legacy);
        validate_runtime_plugin_package_coordinate_segment("package_id", value, &mut optimized);
        assert_eq!(optimized, legacy, "{value:?}");
    }
}

#[test]
fn optimization_batch_20260829bb_runtime329_coordinate_validation_uses_length_check() {
    let source = include_str!("../segment.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(production.matches("value.trim()").count(), 1);
    assert!(production.contains("trimmed.len() != value.len()"));
    assert!(!production.contains("value.trim() != value"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bb_runtime329_single_trim_package_coordinate_bench() {
    let value = format!("plugin_{}", "a".repeat(SEGMENT_BYTES - "plugin_".len()));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&value, false));
            optimized_samples.push(measure(&value, true));
        } else {
            optimized_samples.push(measure(&value, true));
            legacy_samples.push(measure(&value, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME329_SINGLE_TRIM_PACKAGE_COORDINATE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} segment_bytes={SEGMENT_BYTES} \
legacy_full_segment_scans=2 optimized_full_segment_scans=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
