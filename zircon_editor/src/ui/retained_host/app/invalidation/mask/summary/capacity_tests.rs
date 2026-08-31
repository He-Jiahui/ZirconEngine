use std::hint::black_box;
use std::time::Instant;

use super::{HostInvalidationMask, INVALIDATION_SUMMARY_NAME_COUNT};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 47_663;

#[test]
fn optimization_batch_20260826ex_editor139_capacity_preserves_all_summary_names() {
    let mask = all_invalidation_reasons();

    assert_eq!(
        mask.summary(),
        "layout|tree-structure|presentation-data|paint-only|pointer-hover|viewport-image|hit-test|window-metrics|render|shell-content|workbench-projection"
    );
    assert_eq!(INVALIDATION_SUMMARY_NAME_COUNT, 11);
}

#[test]
fn optimization_batch_20260826ex_editor139_invalidation_summary_reserves_fixed_name_count() {
    let source = include_str!("../summary.rs");
    assert!(source.contains("const INVALIDATION_SUMMARY_NAME_COUNT: usize = 11;"));
    assert!(source.contains("Vec::with_capacity(INVALIDATION_SUMMARY_NAME_COUNT)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ex_editor139_invalidation_summary_capacity_bench() {
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
        "EDITOR139_INVALIDATION_SUMMARY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} names_per_build={INVALIDATION_SUMMARY_NAME_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn all_invalidation_reasons() -> HostInvalidationMask {
    HostInvalidationMask::LAYOUT
        .union(HostInvalidationMask::TREE_STRUCTURE)
        .union(HostInvalidationMask::PRESENTATION_DATA)
        .union(HostInvalidationMask::PAINT_ONLY)
        .union(HostInvalidationMask::POINTER_HOVER)
        .union(HostInvalidationMask::VIEWPORT_IMAGE)
        .union(HostInvalidationMask::HIT_TEST)
        .union(HostInvalidationMask::WINDOW_METRICS)
        .union(HostInvalidationMask::RENDER)
        .union(HostInvalidationMask::SHELL_CONTENT)
        .union(HostInvalidationMask::WORKBENCH_PROJECTION)
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut names = if reserve {
            Vec::with_capacity(INVALIDATION_SUMMARY_NAME_COUNT)
        } else {
            Vec::new()
        };
        for name in [
            "layout",
            "tree-structure",
            "presentation-data",
            "paint-only",
            "pointer-hover",
            "viewport-image",
            "hit-test",
            "window-metrics",
            "render",
            "shell-content",
            "workbench-projection",
        ] {
            names.push(black_box(name));
        }
        checksum ^= black_box(names.len() ^ names.capacity());
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
