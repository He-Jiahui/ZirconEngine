use std::{hint::black_box, time::Instant};

use super::*;
use crate::ui::asset_editor::{
    UiAssetEditorExternalEffect, UiAssetEditorSourceCursorSnapshot,
    UiAssetEditorUndoExternalEffects, UiDesignerSelectionModel,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826av_replay_artifact_borrowed_undo_preserves_initial_source() {
    let (stack, current_source) = source_edit_stack(3, 16);

    let (initial_source, error) = reconstruct_initial_source_from_stack(&current_source, &stack);

    assert_eq!(initial_source, "initial");
    assert_eq!(error, None);
    assert_eq!(stack.undo_transitions_rev().count(), 3);
    assert!(stack.can_undo());
    assert!(!stack.can_redo());
}

#[test]
fn optimization_batch_20260826av_replay_artifact_borrows_undo_transitions() {
    let replay_source = include_str!("../replay_artifact.rs");
    let undo_source = include_str!("../../undo_stack.rs");
    let reconstruct = bounded_source(
        replay_source,
        "fn reconstruct_initial_source_from_stack(",
        "fn replay_record_summary(",
    );
    let iterator = bounded_source(
        undo_source,
        "pub(crate) fn undo_transitions_rev(",
        "pub fn replay_records(",
    );

    assert!(reconstruct.contains("undo_stack.undo_transitions_rev()"));
    assert!(!reconstruct.contains("undo_stack.clone()"));
    assert!(!reconstruct.contains("undo_record()"));
    assert!(iterator.contains("self.undo_stack.iter().rev()"));
    assert!(iterator.contains("&entry.undo"));
    assert!(!iterator.contains(".clone()"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826av_replay_artifact_borrowed_undo_p95() {
    const EDIT_COUNT: usize = 2_048;
    const EFFECT_SOURCE_BYTES: usize = 1_024;
    const RECONSTRUCTIONS: usize = 4;
    let (stack, current_source) = source_edit_stack(EDIT_COUNT, EFFECT_SOURCE_BYTES);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(RECONSTRUCTIONS, || {
                legacy_reconstruct(&current_source, &stack)
            }));
            optimized_ns.push(measure_ns(RECONSTRUCTIONS, || {
                reconstruct_initial_source_from_stack(&current_source, &stack)
                    .0
                    .len()
            }));
        } else {
            optimized_ns.push(measure_ns(RECONSTRUCTIONS, || {
                reconstruct_initial_source_from_stack(&current_source, &stack)
                    .0
                    .len()
            }));
            legacy_ns.push(measure_ns(RECONSTRUCTIONS, || {
                legacy_reconstruct(&current_source, &stack)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
        "borrowed undo reconstruction P95 must be at least 80% below stack cloning: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_REPLAY_ARTIFACT_BORROWED_UNDO_BENCH_V1 edits={EDIT_COUNT} effect_source_bytes={EFFECT_SOURCE_BYTES} reconstructions_per_sample={RECONSTRUCTIONS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_stack_clones_per_sample={RECONSTRUCTIONS} optimized_stack_clones_per_sample=0 legacy_transition_clones_per_sample={} optimized_transition_clones_per_sample=0 legacy_deep_effect_bytes_cloned_per_sample={} optimized_deep_effect_bytes_cloned_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        EDIT_COUNT * RECONSTRUCTIONS,
        EDIT_COUNT * EFFECT_SOURCE_BYTES * 3 * RECONSTRUCTIONS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn source_edit_stack(
    edit_count: usize,
    effect_source_bytes: usize,
) -> (UiAssetEditorUndoStack, String) {
    let mut stack = UiAssetEditorUndoStack::default();
    let mut source = "initial".to_string();
    let effect_source = "x".repeat(effect_source_bytes);
    for index in 0..edit_count {
        let before_source = source.clone();
        source.push('x');
        stack.push_edit(
            format!("edit-{index}"),
            None,
            None,
            before_source,
            UiDesignerSelectionModel::default(),
            UiAssetEditorSourceCursorSnapshot::default(),
            None,
            None,
            source.clone(),
            UiDesignerSelectionModel::default(),
            UiAssetEditorSourceCursorSnapshot::default(),
            None,
            None,
            UiAssetEditorUndoExternalEffects {
                undo: vec![UiAssetEditorExternalEffect::RestoreAssetSource {
                    asset_id: format!("asset-{index}"),
                    source: effect_source.clone(),
                }],
                redo: vec![UiAssetEditorExternalEffect::UpsertAssetSource {
                    asset_id: format!("asset-{index}"),
                    source: effect_source.clone(),
                }],
            },
        );
    }
    (stack, source)
}

fn legacy_reconstruct(current_source: &str, stack: &UiAssetEditorUndoStack) -> usize {
    let mut source = current_source.to_string();
    let mut undo_stack = stack.clone();
    while let Some(record) = undo_stack.undo_record() {
        record
            .transition
            .apply_to_source(&mut source)
            .expect("valid undo source replay");
    }
    black_box(source).len()
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
