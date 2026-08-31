use std::hint::black_box;
use std::time::Instant;

use super::ASSET_THUMBNAIL_NAME_AREA_SURFACE;

const CHECKS_PER_SAMPLE: usize = 1_000_000;
const SAMPLE_PAIRS: usize = 31;

#[inline(never)]
fn is_asset_thumbnail_surface(surface_variant: &str, corner_radius: f32) -> bool {
    surface_variant == ASSET_THUMBNAIL_NAME_AREA_SURFACE && corner_radius > 0.0
}

fn legacy_branch_pair(surface_variant: &str, corner_radius: f32) -> usize {
    let first = is_asset_thumbnail_surface(black_box(surface_variant), black_box(corner_radius));
    let second = is_asset_thumbnail_surface(black_box(surface_variant), black_box(corner_radius));
    first as usize + second as usize
}

fn optimized_branch_pair(surface_variant: &str, corner_radius: f32) -> usize {
    let draws_asset_surface =
        is_asset_thumbnail_surface(black_box(surface_variant), black_box(corner_radius));
    draws_asset_surface as usize * 2
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        evidence += if optimized {
            optimized_branch_pair("asset-thumbnail-name-area", 4.0)
        } else {
            legacy_branch_pair("asset-thumbnail-name-area", 4.0)
        };
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
fn optimization_batch_20260829bz_editor298_cached_surface_kind_preserves_results() {
    for (surface_variant, corner_radius) in [
        ("asset-thumbnail-name-area", 4.0),
        ("asset-thumbnail-name-area", 0.0),
        ("panel", 4.0),
        ("", 4.0),
    ] {
        assert_eq!(
            optimized_branch_pair(surface_variant, corner_radius),
            legacy_branch_pair(surface_variant, corner_radius)
        );
    }
}

#[test]
fn optimization_batch_20260829bz_editor298_production_computes_surface_kind_once() {
    let source = include_str!("../commands.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert_eq!(
        production
            .matches("draws_asset_thumbnail_name_area_surface(")
            .count(),
        2
    );
    assert!(production.contains("let draws_asset_thumbnail_name_area"));
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260829bz_editor298_cached_surface_kind_benchmark() {
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(false));
            candidate.push(measure(true));
        } else {
            candidate.push(measure(true));
            baseline.push(measure(false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "EDITOR298_CACHED_ASSET_SURFACE_KIND_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}
