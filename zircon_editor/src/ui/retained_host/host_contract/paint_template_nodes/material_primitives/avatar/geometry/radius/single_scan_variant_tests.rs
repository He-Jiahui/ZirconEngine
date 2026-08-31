use std::hint::black_box;
use std::time::Instant;

use super::{avatar_shape_variant, AvatarShapeVariant};

const CHECKS_PER_SAMPLE: usize = 4096;
const SAMPLE_PAIRS: usize = 31;
const VARIANT_BYTES: usize = 2048;

fn legacy_contains(value: &str, expected: &str) -> bool {
    value
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| part.eq_ignore_ascii_case(expected))
}

fn legacy_avatar_shape_variant(value: &str) -> AvatarShapeVariant {
    if legacy_contains(value, "square") {
        AvatarShapeVariant::Square
    } else if legacy_contains(value, "rounded") {
        AvatarShapeVariant::Rounded
    } else {
        AvatarShapeVariant::Circular
    }
}

fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut shape = AvatarShapeVariant::Circular;
    for _ in 0..CHECKS_PER_SAMPLE {
        shape = if optimized {
            avatar_shape_variant(black_box(value))
        } else {
            legacy_avatar_shape_variant(black_box(value))
        };
    }
    black_box(shape);
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
fn optimization_batch_20260829bt_editor292_avatar_shape_variants_preserve_results() {
    for value in [
        "",
        "rounded",
        "SQUARE",
        "rounded/square",
        "square rounded",
        "roundedish",
        "\u{4f8b}",
    ] {
        assert_eq!(
            avatar_shape_variant(value),
            legacy_avatar_shape_variant(value),
            "{value:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bt_editor292_avatar_shape_uses_one_scan() {
    let source = include_str!("../radius.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains("for part in component_variant.split"));
    assert!(production.contains("return AvatarShapeVariant::Square"));
    assert!(!production.contains("component_variant_contains"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bt_editor292_single_scan_avatar_shape_bench() {
    let value = "x".repeat(VARIANT_BYTES);
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
        "EDITOR292_SINGLE_SCAN_AVATAR_SHAPE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} variant_bytes={VARIANT_BYTES} baseline_variant_scans=2 candidate_variant_scans=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
