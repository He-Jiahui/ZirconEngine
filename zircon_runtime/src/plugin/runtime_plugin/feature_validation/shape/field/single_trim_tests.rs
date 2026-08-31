use std::hint::black_box;
use std::time::Instant;

use super::validate_runtime_plugin_feature_field;

const CHECKS_PER_SAMPLE: usize = 16_384;
const FIELD_BYTES: usize = 4096;
const SAMPLE_PAIRS: usize = 31;

fn legacy_validate(field_name: &str, value: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() || value.trim() != value {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must be non-empty and trimmed"
        ));
    }
}

fn measure(value: &str, optimized: bool) -> u128 {
    let mut diagnostics = Vec::new();
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            validate_runtime_plugin_feature_field(
                "display_name",
                black_box(value),
                &mut diagnostics,
            );
        } else {
            legacy_validate("display_name", black_box(value), &mut diagnostics);
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
fn optimization_batch_20260829bd_runtime331_single_trim_feature_preserves_diagnostics() {
    for value in [
        "Shadow Feature",
        "",
        " ",
        " Shadow Feature",
        "Shadow Feature ",
    ] {
        let mut legacy = Vec::new();
        let mut optimized = Vec::new();
        legacy_validate("display_name", value, &mut legacy);
        validate_runtime_plugin_feature_field("display_name", value, &mut optimized);
        assert_eq!(optimized, legacy, "{value:?}");
    }
}

#[test]
fn optimization_batch_20260829bd_runtime331_feature_validation_uses_length_check() {
    let source = include_str!("../field.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(production.matches("value.trim()").count(), 1);
    assert!(production.contains("trimmed.len() != value.len()"));
    assert!(!production.contains("value.trim() != value"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bd_runtime331_single_trim_feature_field_bench() {
    let value = format!("Feature {}", "x".repeat(FIELD_BYTES - "Feature ".len()));
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
        "RUNTIME331_SINGLE_TRIM_FEATURE_FIELD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} field_bytes={FIELD_BYTES} \
legacy_full_field_comparisons=1 optimized_full_field_comparisons=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
