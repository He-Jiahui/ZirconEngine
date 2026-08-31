use std::hint::black_box;
use std::time::Instant;

use super::compare_material_name_text;

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const NAME_BYTES: usize = 4096;

fn legacy_compare(left: &str, right: &str) -> std::cmp::Ordering {
    left.bytes()
        .map(|byte| byte.to_ascii_lowercase())
        .cmp(right.bytes().map(|byte| byte.to_ascii_lowercase()))
        .then_with(|| left.cmp(right))
}

fn measure(left: &str, right: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut ordering = std::cmp::Ordering::Equal;
    for _ in 0..CHECKS_PER_SAMPLE {
        ordering = if optimized {
            compare_material_name_text(black_box(left), black_box(right))
        } else {
            legacy_compare(black_box(left), black_box(right))
        };
    }
    black_box(ordering);
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
fn optimization_batch_20260829bo_runtime342_fast_material_name_case_preserves_order() {
    for (left, right) in [
        ("Metal", "metal"),
        ("alpha", "beta"),
        ("same", "same"),
        ("\u{4f8b}", "\u{4f8b}"),
    ] {
        assert_eq!(
            compare_material_name_text(left, right),
            legacy_compare(left, right),
            "{left:?} vs {right:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bo_runtime342_fast_material_name_case_uses_borrowed_path() {
    let source = include_str!("../sort_order.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains("let mut tie_break = Ordering::Equal;"));
    assert!(production.contains("tie_break = left_byte.cmp(&right_byte);"));
    assert!(!production.contains(".then_with(|| left.cmp(right))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bo_runtime342_fast_material_name_case_bench() {
    let left = "M".repeat(NAME_BYTES);
    let right = "m".repeat(NAME_BYTES);
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&left, &right, false));
            candidate.push(measure(&left, &right, true));
        } else {
            candidate.push(measure(&left, &right, true));
            baseline.push(measure(&left, &right, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME342_FAST_MATERIAL_NAME_CASE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} name_bytes={NAME_BYTES} baseline_name_scans=2 candidate_name_scans=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
