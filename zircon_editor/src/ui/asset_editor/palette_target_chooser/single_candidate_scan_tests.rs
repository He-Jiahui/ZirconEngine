use std::{hint::black_box, time::Instant};

use super::*;
use crate::ui::asset_editor::palette::{PaletteInsertMode, UiAssetPaletteInsertionPlacement};
use crate::ui::asset_editor::tree::palette_drop::UiAssetPaletteInsertPlan;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826aw_palette_chooser_preserves_manual_sticky_and_invalid_selection() {
    let previous_resolution = resolution(8, 6);
    let mut next_resolution = previous_resolution.clone();
    next_resolution.selected_index = 0;
    let previous = UiAssetPaletteTargetChooser::new(previous_resolution, true, false);

    let (next, changed) = reconcile_palette_target_chooser(Some(previous), Some(next_resolution));
    let next = next.expect("reconciled chooser");
    assert_eq!(next.resolution().selected_index, 6);
    assert!(next.manual_selection());
    assert!(!changed);

    let sticky_resolution = resolution(8, 4);
    let sticky = UiAssetPaletteTargetChooser::new(sticky_resolution.clone(), false, true);
    let mut changed_resolution = sticky_resolution;
    changed_resolution.candidates[0].detail.push_str("-changed");
    let (retained, changed) =
        reconcile_palette_target_chooser(Some(sticky.clone()), Some(changed_resolution));
    assert_eq!(retained, Some(sticky));
    assert!(!changed);

    let invalid_previous = UiAssetPaletteTargetChooser::new(resolution(8, usize::MAX), true, false);
    let (next, _) =
        reconcile_palette_target_chooser(Some(invalid_previous), Some(resolution(8, 0)));
    let next = next.expect("fallback chooser");
    assert_eq!(next.resolution().selected_index, 0);
    assert!(!next.manual_selection());
}

#[test]
fn optimization_batch_20260826aw_palette_chooser_scans_candidate_set_once() {
    let source = include_str!("../palette_target_chooser.rs");
    let reconcile = bounded_source(
        source,
        "pub(super) fn reconcile_palette_target_chooser(",
        "fn same_candidate_set(",
    );

    assert_eq!(reconcile.matches("same_candidate_set(").count(), 1);
    assert!(reconcile.contains("previous_ref.resolution().selected_index"));
    assert!(reconcile.contains("previous_ref.selected_target().is_some()"));
    assert!(!reconcile.contains(".position("));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826aw_palette_chooser_single_candidate_scan_p95() {
    const CANDIDATES: usize = 16_384;
    const RECONCILIATIONS: usize = 64;
    let previous =
        UiAssetPaletteTargetChooser::new(resolution(CANDIDATES, CANDIDATES - 1), true, true);
    let next = resolution(CANDIDATES, 0);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(RECONCILIATIONS, || {
                legacy_reconciled_index(black_box(&previous), black_box(&next))
            }));
            optimized_ns.push(measure_ns(RECONCILIATIONS, || {
                optimized_reconciled_index(black_box(&previous), black_box(&next))
            }));
        } else {
            optimized_ns.push(measure_ns(RECONCILIATIONS, || {
                optimized_reconciled_index(black_box(&previous), black_box(&next))
            }));
            legacy_ns.push(measure_ns(RECONCILIATIONS, || {
                legacy_reconciled_index(black_box(&previous), black_box(&next))
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "single candidate scan P95 must be at least 50% below repeated comparison and lookup: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_PALETTE_CHOOSER_SINGLE_CANDIDATE_SCAN_BENCH_V1 candidates={CANDIDATES} reconciliations_per_sample={RECONCILIATIONS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_candidate_visits_per_sample={} optimized_candidate_visits_per_sample={} legacy_candidate_scans_per_reconciliation=3 optimized_candidate_scans_per_reconciliation=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        CANDIDATES * RECONCILIATIONS * 3,
        CANDIDATES * RECONCILIATIONS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn resolution(candidate_count: usize, selected_index: usize) -> UiAssetPaletteDragResolution {
    UiAssetPaletteDragResolution {
        candidates: (0..candidate_count)
            .map(|index| UiAssetPaletteDragTarget {
                preview_index: Some(index),
                plan: UiAssetPaletteInsertPlan {
                    node_id: format!("node-{index:05}"),
                    mode: PaletteInsertMode::Child,
                    label: format!("Insert {index:05}"),
                    placement: UiAssetPaletteInsertionPlacement::default(),
                },
                key: format!("candidate-{index:05}"),
                detail: format!("detail-{index:05}"),
            })
            .collect(),
        selected_index,
        requires_confirmation: true,
    }
}

fn optimized_reconciled_index(
    previous: &UiAssetPaletteTargetChooser,
    next: &UiAssetPaletteDragResolution,
) -> usize {
    let same_candidates = same_candidate_set(previous.resolution(), next);
    if same_candidates && previous.manual_selection() && previous.selected_target().is_some() {
        previous.resolution().selected_index
    } else {
        next.selected_index
    }
}

fn legacy_reconciled_index(
    previous: &UiAssetPaletteTargetChooser,
    next: &UiAssetPaletteDragResolution,
) -> usize {
    if previous.sticky() && !same_candidate_set(black_box(previous.resolution()), black_box(next)) {
        return previous.resolution().selected_index;
    }
    if same_candidate_set(black_box(previous.resolution()), black_box(next))
        && previous.manual_selection()
    {
        if let Some(previous_target) = previous.selected_target() {
            return next
                .candidates
                .iter()
                .position(|candidate| {
                    candidate.key == previous_target.key
                        && candidate.detail == previous_target.detail
                })
                .unwrap_or(next.selected_index);
        }
    }
    next.selected_index
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
