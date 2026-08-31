use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828in_editor_replay_state_reuses_nested_allocations() {
    let source = benchmark_selection("source");
    let mut target = benchmark_selection("target");
    let siblings_allocation = target.sibling_node_ids.as_ptr();
    let primary_allocation = target
        .primary_node_id
        .as_ref()
        .expect("target primary")
        .as_ptr();
    let first_sibling_allocation = target.sibling_node_ids[0].as_ptr();

    reuse_selection_state(&mut target, &source);

    assert_eq!(target, source);
    assert_eq!(target.sibling_node_ids.as_ptr(), siblings_allocation);
    assert_eq!(
        target
            .primary_node_id
            .as_ref()
            .expect("source primary")
            .as_ptr(),
        primary_allocation
    );
    assert_eq!(
        target.sibling_node_ids[0].as_ptr(),
        first_sibling_allocation
    );

    let source_cursor = UiAssetEditorSourceCursorSnapshot {
        byte_offset: 4096,
        anchor_node_id: Some(fixed_identifier("source-anchor", 256)),
        line_offset: 64,
    };
    let mut target_cursor = UiAssetEditorSourceCursorSnapshot {
        byte_offset: 0,
        anchor_node_id: Some(fixed_identifier("target-anchor", 256)),
        line_offset: 0,
    };
    let anchor_allocation = target_cursor
        .anchor_node_id
        .as_ref()
        .expect("target anchor")
        .as_ptr();
    reuse_source_cursor(&mut target_cursor, &source_cursor);
    assert_eq!(target_cursor, source_cursor);
    assert_eq!(
        target_cursor
            .anchor_node_id
            .as_ref()
            .expect("source anchor")
            .as_ptr(),
        anchor_allocation
    );

    let source_option = Some(fixed_identifier("source-option", 256));
    let mut target_option = Some(fixed_identifier("target-option", 256));
    let option_allocation = target_option.as_ref().expect("target option").as_ptr();
    reuse_optional_string(&mut target_option, &source_option);
    assert_eq!(target_option, source_option);
    assert_eq!(
        target_option.as_ref().expect("source option").as_ptr(),
        option_allocation
    );
}

#[test]
fn optimization_batch_20260828in_editor_replay_uses_field_level_clone_from() {
    let source = include_str!("../replay_workspace.rs");
    let apply = source
        .split("pub fn apply_to_workspace")
        .nth(1)
        .and_then(|body| body.split("fn reuse_selection_state").next())
        .expect("workspace replay apply path");

    assert!(apply.contains("reuse_selection_state("));
    assert!(apply.contains("reuse_source_cursor("));
    assert_eq!(apply.matches("reuse_optional_string(").count(), 2);
    assert!(!apply.contains("workspace.selection = self.selection.clone()"));
    assert!(!apply.contains("workspace.source_cursor = self.source_cursor.clone()"));
    assert!(source.contains("target.sibling_node_ids.clone_from(&source.sibling_node_ids)"));
    assert!(source.contains("target.anchor_node_id.clone_from(&source.anchor_node_id)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828in_editor_reused_undo_replay_state_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 4096;
    let first = benchmark_selection("first-");
    let second = benchmark_selection("second");

    let mut warm = benchmark_selection("warm--");
    legacy_replace_selection(&mut warm, &first);
    reuse_selection_state(&mut warm, &second);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_target = benchmark_selection("legacy");
        let optimized_target = benchmark_selection("opt---");
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_selection_updates(
                legacy_target,
                &first,
                &second,
                ITERATIONS,
                legacy_replace_selection,
            ));
            optimized_samples.push(measure_selection_updates(
                optimized_target,
                &first,
                &second,
                ITERATIONS,
                reuse_selection_state,
            ));
        } else {
            optimized_samples.push(measure_selection_updates(
                optimized_target,
                &first,
                &second,
                ITERATIONS,
                reuse_selection_state,
            ));
            legacy_samples.push(measure_selection_updates(
                legacy_target,
                &first,
                &second,
                ITERATIONS,
                legacy_replace_selection,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR232_REUSED_UNDO_REPLAY_STATE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_selection(seed: &str) -> UiDesignerSelectionModel {
    UiDesignerSelectionModel {
        primary_node_id: Some(fixed_identifier(seed, 64)),
        sibling_node_ids: (0..128)
            .map(|index| fixed_identifier(&format!("{seed}-{index:03}"), 64))
            .collect(),
        parent_node_id: Some(fixed_identifier(&format!("{seed}-parent"), 64)),
        mount: Some(fixed_identifier(&format!("{seed}-mount"), 64)),
    }
}

fn fixed_identifier(seed: &str, bytes: usize) -> String {
    let mut value = String::with_capacity(bytes);
    while value.len() < bytes {
        value.push_str(seed);
    }
    value.truncate(bytes);
    value
}

fn legacy_replace_selection(
    target: &mut UiDesignerSelectionModel,
    source: &UiDesignerSelectionModel,
) {
    *target = source.clone();
}

fn measure_selection_updates(
    mut target: UiDesignerSelectionModel,
    first: &UiDesignerSelectionModel,
    second: &UiDesignerSelectionModel,
    iterations: usize,
    update: fn(&mut UiDesignerSelectionModel, &UiDesignerSelectionModel),
) -> u128 {
    let started = Instant::now();
    for iteration in 0..iterations {
        let source = if iteration % 2 == 0 { first } else { second };
        update(black_box(&mut target), black_box(source));
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(target);
    elapsed
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
