use std::hint::black_box;
use std::time::Instant;

use super::module_field_is_valid;

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const FIELD_PADDING_BYTES: usize = 4096;

fn legacy_module_field_is_valid(value: &str) -> bool {
    !value.trim().is_empty() && value.trim() == value
}

fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut valid = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        valid += if optimized {
            module_field_is_valid(black_box(value)) as usize
        } else {
            legacy_module_field_is_valid(black_box(value)) as usize
        };
    }
    black_box(valid);
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
fn runtime_hotpath_batch_runtime340_341_module_fields_preserve_results() {
    for value in ["module", " module", "module ", " ", "", "\u{4f8b}"] {
        assert_eq!(
            module_field_is_valid(value),
            legacy_module_field_is_valid(value),
            "{value:?}"
        );
    }
}

#[test]
fn runtime_hotpath_batch_runtime340_341_module_field_trims_once() {
    let source = include_str!("../runtime_core.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains("let trimmed = value.trim();"));
    assert!(production.contains("module_field_is_valid(value)"));
    assert_eq!(production.matches(".trim()").count(), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_hotpath_batch_runtime340_341_single_trim_module_field_bench() {
    let value = format!(
        "{}module{}",
        " ".repeat(FIELD_PADDING_BYTES / 2),
        " ".repeat(FIELD_PADDING_BYTES / 2)
    );
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&value, false));
            candidate.push(measure(&value, true));
        } else {
            candidate.push(measure(&value, true));
            baseline.push(measure(&value, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME341_SINGLE_TRIM_MODULE_FIELD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} field_padding_bytes={FIELD_PADDING_BYTES} baseline_trim_calls=2 candidate_trim_calls=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
