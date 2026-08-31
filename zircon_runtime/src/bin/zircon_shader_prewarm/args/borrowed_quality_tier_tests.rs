use std::hint::black_box;
use std::time::Instant;

use super::quality_tier_variant;

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const TOKEN_BYTES: usize = 4096;

fn legacy_quality_tier_variant(value: &str) -> u8 {
    match value.trim().to_ascii_lowercase().as_str() {
        "low" => 1,
        "medium" => 2,
        "high" => 3,
        "ultra" => 4,
        "all" => 5,
        _ => 0,
    }
}

fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut variant = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        variant = if optimized {
            quality_tier_variant(black_box(value))
        } else {
            legacy_quality_tier_variant(black_box(value))
        };
    }
    black_box(variant);
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
fn optimization_batch_20260829bp_runtime343_quality_tier_variants_preserve_results() {
    for value in [
        "low", " MEDIUM ", "High", "ULTRA", "all", "unknown", "\u{4f8b}",
    ] {
        assert_eq!(
            quality_tier_variant(value),
            legacy_quality_tier_variant(value),
            "{value:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bp_runtime343_quality_tier_uses_borrowed_classifier() {
    let source = include_str!("../args.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains("fn quality_tier_variant(value: &str) -> u8"));
    assert!(production.contains("value.eq_ignore_ascii_case"));
    let parser = production
        .split_once("fn parse_quality_tier")
        .expect("quality parser")
        .1
        .split_once("fn parse_geometry_source")
        .expect("geometry parser boundary")
        .0;
    assert!(!parser.contains("to_ascii_lowercase"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bp_runtime343_borrowed_quality_tier_bench() {
    let value = "x".repeat(TOKEN_BYTES);
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
        "RUNTIME343_BORROWED_QUALITY_TIER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} token_bytes={TOKEN_BYTES} baseline_lowercase_allocations={CHECKS_PER_SAMPLE} candidate_lowercase_allocations=0 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
