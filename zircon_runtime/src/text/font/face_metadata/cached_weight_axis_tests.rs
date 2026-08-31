use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;

const AXIS_COUNT: usize = 512;
const CHECKS_PER_SAMPLE: usize = 4096;
const SAMPLE_PAIRS: usize = 31;

fn legacy_effective_variations(
    metadata: &FontFaceMetadata,
    variations: &VariationCoords,
    font_weight: Option<u16>,
) -> VariationCoords {
    let mut weighted = variations.clone();
    if let Some(font_weight) = font_weight {
        if metadata.axes.iter().any(|axis| axis.tag == WEIGHT_AXIS_TAG) {
            weighted.0.push((WEIGHT_AXIS_TAG, f32::from(font_weight)));
        }
    }
    let weighted = canonical_variation_coords(&weighted).expect("valid fixture coordinates");
    VariationCoords(
        weighted
            .0
            .into_iter()
            .filter_map(|(tag, value)| {
                let axis = metadata.axes.iter().find(|axis| axis.tag == tag)?;
                let value =
                    quantized_axis_value(value, axis.min_value, axis.default_value, axis.max_value);
                (value != axis.default_value).then_some((tag, value))
            })
            .collect(),
    )
}

fn fixture_metadata() -> FontFaceMetadata {
    let mut metadata = FontFaceMetadata::unknown([0; 16]);
    metadata.parsed = true;
    let mut axes = (0..AXIS_COUNT - 1)
        .map(|index| FontVariationAxis {
            tag: index as u32,
            min_value: 0.0,
            default_value: 0.0,
            max_value: 1000.0,
        })
        .collect::<Vec<_>>();
    axes.push(FontVariationAxis {
        tag: WEIGHT_AXIS_TAG,
        min_value: 100.0,
        default_value: 400.0,
        max_value: 900.0,
    });
    metadata.axes = Arc::from(axes.into_boxed_slice());
    metadata
}

fn measure(metadata: &FontFaceMetadata, optimized: bool) -> u128 {
    let started = Instant::now();
    let variations = VariationCoords::default();
    let mut evidence = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let result = if optimized {
            metadata.effective_variations(black_box(&variations), Some(700))
        } else {
            legacy_effective_variations(metadata, black_box(&variations), Some(700))
        };
        evidence += result.0.len();
        black_box(result);
    }
    black_box(evidence);
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
fn optimization_batch_20260829bw_runtime350_weight_axis_cache_preserves_results() {
    let metadata = fixture_metadata();
    for (variations, weight) in [
        (VariationCoords::default(), Some(700)),
        (VariationCoords(vec![(WEIGHT_AXIS_TAG, 250.0)]), None),
        (VariationCoords(vec![(WEIGHT_AXIS_TAG, 250.0)]), Some(700)),
    ] {
        assert_eq!(
            metadata.effective_variations(&variations, weight),
            legacy_effective_variations(&metadata, &variations, weight)
        );
    }
}

#[test]
fn optimization_batch_20260829bw_runtime350_weight_axis_lookup_is_reused() {
    let source = include_str!("../face_metadata.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("pub(super) fn effective_variations")
        .expect("variation function")
        .1
        .split_once("fn unknown")
        .expect("unknown boundary")
        .0;
    assert!(function.contains("let weight_axis ="));
    assert!(function.contains("font_weight.and_then"));
    assert!(function.contains("weight_axis.or_else"));
    assert!(!function.contains(".any(|axis| axis.tag == WEIGHT_AXIS_TAG)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bw_runtime350_cached_weight_axis_bench() {
    let metadata = fixture_metadata();
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&metadata, false));
            candidate.push(measure(&metadata, true));
        } else {
            candidate.push(measure(&metadata, true));
            baseline.push(measure(&metadata, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME350_CACHED_WEIGHT_AXIS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} axis_count={AXIS_COUNT} baseline_weight_axis_scans=2 candidate_weight_axis_scans=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
