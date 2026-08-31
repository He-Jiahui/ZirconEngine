use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ia_editor_text_focus_borrows_target_and_moves_control_id() {
    let mut focus = benchmark_focus(4 * 1024);
    let control_allocation = focus.control_id.as_ptr();
    let edit_target_allocation = focus.edit_action_id.as_ptr();

    assert_eq!(
        text_focus_edit_target_id(&focus).as_ptr(),
        edit_target_allocation
    );
    let control_id = take_text_focus_control_id(&mut focus);

    assert_eq!(control_id.as_ptr(), control_allocation);
    assert!(focus.control_id.is_empty());
}

#[test]
fn optimization_batch_20260828ia_editor_text_focus_dispatch_has_no_string_clone() {
    let source = include_str!("../dispatch.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_text_focus_value")
        .nth(1)
        .and_then(|body| body.split("fn text_focus_edit_target_id").next())
        .expect("text focus dispatch implementation");

    assert!(dispatch.contains("text_focus_edit_target_id(&focus)"));
    assert!(dispatch.contains("take_text_focus_control_id(&mut focus)"));
    assert!(!dispatch.contains("focus.control_id.clone()"));
    assert!(!dispatch.contains("focus.edit_target_id()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ia_editor_owned_text_focus_control_id_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 256;

    black_box(legacy_text_focus_dispatch_parts(
        benchmark_focus(64 * 1024),
        "edit-target",
    ));
    black_box(owned_text_focus_dispatch_parts(
        benchmark_focus(64 * 1024),
        "edit-target",
    ));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| benchmark_focus(64 * 1024))
            .collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS)
            .map(|_| benchmark_focus(64 * 1024))
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_focuses(
                legacy_inputs,
                legacy_text_focus_dispatch_parts,
            ));
            optimized_samples.push(measure_focuses(
                optimized_inputs,
                owned_text_focus_dispatch_parts,
            ));
        } else {
            optimized_samples.push(measure_focuses(
                optimized_inputs,
                owned_text_focus_dispatch_parts,
            ));
            legacy_samples.push(measure_focuses(
                legacy_inputs,
                legacy_text_focus_dispatch_parts,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR219_OWNED_TEXT_FOCUS_CONTROL_ID_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_focus(field_bytes: usize) -> HostTextInputFocusData {
    HostTextInputFocusData {
        control_id: "control".repeat(field_bytes / 7),
        dispatch_kind: "commit_only".into(),
        action_id: "action".repeat(field_bytes / 6),
        edit_action_id: "edit-target".repeat(field_bytes / 11),
        commit_action_id: "commit".repeat(field_bytes / 6),
        value_text: "value".repeat(field_bytes / 5),
        edit_frame: Default::default(),
    }
}

fn legacy_text_focus_dispatch_parts(
    focus: HostTextInputFocusData,
    target_id: &str,
) -> (SharedString, bool) {
    let control_id = focus.control_id.clone();
    let is_commit_target = target_id == focus.edit_target_id();
    (control_id, is_commit_target)
}

fn owned_text_focus_dispatch_parts(
    mut focus: HostTextInputFocusData,
    target_id: &str,
) -> (SharedString, bool) {
    let is_commit_target = target_id == text_focus_edit_target_id(&focus);
    let control_id = take_text_focus_control_id(&mut focus);
    (control_id, is_commit_target)
}

fn measure_focuses(
    focuses: Vec<HostTextInputFocusData>,
    mut dispatch: impl FnMut(HostTextInputFocusData, &str) -> (SharedString, bool),
) -> u128 {
    let started = Instant::now();
    for focus in focuses {
        black_box(dispatch(black_box(focus), black_box("edit-target")));
    }
    started.elapsed().as_nanos()
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
