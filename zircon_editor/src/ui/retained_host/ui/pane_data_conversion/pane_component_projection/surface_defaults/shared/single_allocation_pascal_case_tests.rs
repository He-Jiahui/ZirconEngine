use std::hint::black_box;
use std::time::Instant;

use super::pascal_case;

const SAMPLE_PAIRS: usize = 31;
const CONVERSIONS_PER_SAMPLE: usize = 100_000;
const VALUE: &str = "surface_variant_with_a_long_component_identity_for_repeated_projection";

#[test]
fn optimization_batch_20260829iv_editor240_pascal_case_preserves_ascii_and_unicode_results() {
    for (input, expected) in [
        ("", ""),
        ("success", "Success"),
        ("Success", "Success"),
        ("7zip", "7zip"),
        ("eclair", "Eclair"),
        ("\u{00e9}clair", "\u{00e9}clair"),
    ] {
        assert_eq!(pascal_case(input), expected);
    }
}

#[test]
fn optimization_batch_20260829iv_editor240_pascal_case_builds_one_output_string() {
    let source = include_str!("../shared.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("pub(super) fn pascal_case")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn alert_color_severity").next())
        .expect("pascal-case helper");

    assert!(body.contains("String::with_capacity(value.len())"));
    assert!(body.contains("result.push(first.to_ascii_uppercase())"));
    assert!(body.contains("result.push_str(characters.as_str())"));
    assert!(!body.contains("to_string() +"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829iv_editor240_single_allocation_pascal_case_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR240_SINGLE_ALLOCATION_PASCAL_CASE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
conversions_per_sample={CONVERSIONS_PER_SAMPLE} value_bytes={} \
legacy_allocations_per_conversion=2 optimized_allocations_per_conversion=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        VALUE.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + characters.as_str()
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for iteration in 0..CONVERSIONS_PER_SAMPLE {
        let converted = if optimized {
            pascal_case(black_box(VALUE))
        } else {
            legacy_pascal_case(black_box(VALUE))
        };
        checksum ^= black_box(converted.len().wrapping_add(iteration));
    }
    black_box(checksum);
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
