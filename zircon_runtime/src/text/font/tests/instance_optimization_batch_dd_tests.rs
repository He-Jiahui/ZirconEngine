use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{canonical_variation_coords, FontInstanceError};
use crate::text::VariationCoords;

const SAMPLE_PAIRS: usize = 17;
const AXES_PER_SAMPLE: usize = 16_384;

#[test]
fn font_variation_canonicalization_preserves_last_duplicate_and_sorted_order() {
    let input = VariationCoords(vec![(3, 1.0), (1, 2.0), (3, 4.0), (2, -0.0)]);
    assert_eq!(
        canonical_variation_coords(&input).unwrap(),
        VariationCoords(vec![(1, 2.0), (2, 0.0), (3, 4.0)])
    );
}

#[test]
fn optimization_batch_dd_font_variation_canonicalization_matches_tree_baseline() {
    let input = VariationCoords(
        (0..4_096)
            .map(|index| {
                let tag = (index as u32).wrapping_mul(17) % 257;
                (tag, if index % 31 == 0 { -0.0 } else { index as f32 })
            })
            .collect(),
    );
    assert_eq!(
        canonical_variation_coords(&input),
        legacy_canonical_variation_coords(&input.0)
    );
}

#[test]
fn optimization_batch_dd_font_variation_canonicalization_uses_sorted_vector() {
    let production = include_str!("../instance.rs")
        .split_once("#[cfg(test)]")
        .expect("production source and tests must remain separated")
        .0;
    assert!(production.contains("Vec::with_capacity(variations.0.len())"));
    assert!(production.contains("coordinates.sort_by_key(|(tag, _)| *tag)"));
    assert!(production.contains("coordinates.truncate(unique_count)"));
    assert!(!production.contains("let mut coordinates = BTreeMap::new()"));
}

#[test]
fn optimization_batch_dd_font_variation_canonicalization_rejects_non_finite_values() {
    let error = canonical_variation_coords(&VariationCoords(vec![(7, f32::NAN)])).unwrap_err();
    assert_eq!(error, FontInstanceError::NonFiniteCoordinate { tag: 7 });
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_dd_runtime411_font_variation_vector_sort_p95() {
    let input = VariationCoords(
        (0..AXES_PER_SAMPLE)
            .map(|index| {
                let tag = (index as u32).wrapping_mul(0x9e37_79b9);
                (tag, (index % 997) as f32 + 0.5)
            })
            .collect::<Vec<_>>(),
    );

    for _ in 0..3 {
        black_box(legacy_canonical_variation_coords(black_box(&input.0)));
        black_box(canonical_variation_coords(black_box(&input)));
    }

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_canonical_variation_coords(&input.0)));
            optimized.push(measure(|| canonical_variation_coords(&input)));
        } else {
            optimized.push(measure(|| canonical_variation_coords(&input)));
            legacy.push(measure(|| legacy_canonical_variation_coords(&input.0)));
        }
    }

    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME411_FONT_VARIATION_VECTOR_SORT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} axes_per_sample={AXES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_canonical_variation_coords(
    values: &[(u32, f32)],
) -> Result<VariationCoords, FontInstanceError> {
    let mut coordinates = BTreeMap::new();
    for &(tag, value) in values {
        if !value.is_finite() {
            return Err(FontInstanceError::NonFiniteCoordinate { tag });
        }
        coordinates.insert(tag, if value == 0.0 { 0.0 } else { value });
    }
    Ok(VariationCoords(coordinates.into_iter().collect()))
}

fn measure(run: impl FnOnce() -> Result<VariationCoords, FontInstanceError>) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
