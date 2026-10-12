use std::hint::black_box;
use std::time::Instant;

use super::*;

const INVALIDATIONS_PER_SAMPLE: usize = 131_072;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_fm_editor399_borrowed_paint_only_view_preserves_diagnostics() {
    let view =
        ViewInstanceId::new("editor.scene-view#main/viewport/active-document/long-lived-instance");
    let mask = HostInvalidationMask::PAINT_ONLY
        .union(HostInvalidationMask::POINTER_HOVER)
        .union(HostInvalidationMask::VIEWPORT_IMAGE);
    let mut root = HostInvalidationRoot::default();

    root.invalidate_view(&view, mask);

    assert_eq!(root.total_requests, 1);
    assert_eq!(root.paint_only_requests, 1);
    assert_eq!(root.layout_requests, 0);
    assert_eq!(root.presentation_requests, 0);
    assert!(root.take_recompute_transaction().reasons().is_empty());
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fm_editor399_borrowed_paint_only_view_benchmark() {
    let view =
        ViewInstanceId::new("editor.scene-view#main/viewport/active-document/long-lived-instance");
    for _ in 0..4 {
        black_box(measure_legacy(&view));
        black_box(measure_optimized(&view));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_legacy(&view));
            optimized_samples.push(measure_optimized(&view));
        } else {
            optimized_samples.push(measure_optimized(&view));
            legacy_samples.push(measure_legacy(&view));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn measure_legacy(view: &ViewInstanceId) -> u128 {
    let mut root = HostInvalidationRoot::default();
    let started = Instant::now();
    for _ in 0..INVALIDATIONS_PER_SAMPLE {
        root.invalidate_scoped(
            HostInvalidationScope::View(black_box(view).clone()),
            HostInvalidationMask::PAINT_ONLY,
        );
    }
    black_box(root);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(view: &ViewInstanceId) -> u128 {
    let mut root = HostInvalidationRoot::default();
    let started = Instant::now();
    for _ in 0..INVALIDATIONS_PER_SAMPLE {
        root.invalidate_view(black_box(view), HostInvalidationMask::PAINT_ONLY);
    }
    black_box(root);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR399_BORROWED_PAINT_ONLY_VIEW_BENCH_V1 sample_pairs={SAMPLE_PAIRS} invalidations_per_sample={INVALIDATIONS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
        "borrowed paint-only view invalidation must reduce P95 by at least 25%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
