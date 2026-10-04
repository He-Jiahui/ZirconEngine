use std::hint::black_box;
use std::time::Instant;

use super::{append_badge_geometry_variant_tokens, append_variant_token, pascal_case};

fn legacy(variant: &mut String, overlap: &str, vertical: &str, horizontal: &str) {
    append_variant_token(variant, overlap);
    append_variant_token(variant, &format!("overlap{}", pascal_case(overlap)));
    append_variant_token(variant, vertical);
    append_variant_token(variant, horizontal);
    append_variant_token(
        variant,
        &format!(
            "anchorOrigin{}{}",
            pascal_case(vertical),
            pascal_case(horizontal)
        ),
    );
    append_variant_token(
        variant,
        &format!(
            "anchorOrigin{}{}{}",
            pascal_case(vertical),
            pascal_case(horizontal),
            pascal_case(overlap)
        ),
    );
}

#[test]
fn optimization_batch_em_cached_badge_variant_case_preserves_tokens() {
    let overlap = "rectangular surface";
    let vertical = "bottom edge";
    let horizontal = "right edge";
    let mut legacy_variant = String::new();
    let mut optimized_variant = String::new();

    legacy(&mut legacy_variant, overlap, vertical, horizontal);
    append_badge_geometry_variant_tokens(&mut optimized_variant, overlap, vertical, horizontal);

    assert_eq!(optimized_variant, legacy_variant);
}

#[test]
fn optimization_batch_em_badge_variant_case_is_derived_once_per_axis() {
    let source = include_str!("../badge.rs");
    let helper = source
        .split("fn append_badge_geometry_variant_tokens(")
        .nth(1)
        .expect("badge geometry variant helper")
        .split("fn badge_variant(")
        .next()
        .expect("bounded badge geometry variant helper");

    assert_eq!(helper.matches("pascal_case(overlap)").count(), 1);
    assert_eq!(helper.matches("pascal_case(vertical)").count(), 1);
    assert_eq!(helper.matches("pascal_case(horizontal)").count(), 1);
}

#[test]
#[ignore = "release-only cached badge variant case benchmark"]
fn optimization_batch_em_reused_badge_variant_case_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const PROJECTIONS_PER_SAMPLE: usize = 4_096;

    fn measure_legacy(overlap: &str, vertical: &str, horizontal: &str) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..PROJECTIONS_PER_SAMPLE {
            let mut variant = String::with_capacity(2_048);
            legacy(&mut variant, overlap, vertical, horizontal);
            checksum = checksum.wrapping_add(variant.len());
            black_box(variant);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(overlap: &str, vertical: &str, horizontal: &str) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..PROJECTIONS_PER_SAMPLE {
            let mut variant = String::with_capacity(2_048);
            append_badge_geometry_variant_tokens(&mut variant, overlap, vertical, horizontal);
            checksum = checksum.wrapping_add(variant.len());
            black_box(variant);
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

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let overlap = "rectangular surface ".repeat(16);
    let vertical = "bottom edge ".repeat(16);
    let horizontal = "right edge ".repeat(16);
    for _ in 0..4 {
        black_box(measure_legacy(&overlap, &vertical, &horizontal));
        black_box(measure_optimized(&overlap, &vertical, &horizontal));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&overlap, &vertical, &horizontal));
            optimized_samples.push(measure_optimized(&overlap, &vertical, &horizontal));
        } else {
            optimized_samples.push(measure_optimized(&overlap, &vertical, &horizontal));
            legacy_samples.push(measure_legacy(&overlap, &vertical, &horizontal));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR375_REUSED_BADGE_VARIANT_CASE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             projections_per_sample={PROJECTIONS_PER_SAMPLE} input_bytes={} \
             pair_order=alternating_legacy_even legacy_pascal_case_calls_per_projection=6 \
             optimized_pascal_case_calls_per_projection=3 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        overlap.len() + vertical.len() + horizontal.len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
        "reused badge variant case conversion must reduce P95 by at least 25%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
