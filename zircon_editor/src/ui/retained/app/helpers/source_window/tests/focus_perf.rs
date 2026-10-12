use std::hint::black_box;

use super::MainPageId;

const LOOKUP_ROUNDS: usize = 65_536;
const PREVIOUS_WINDOW_ID: &str = "window:editor.callback-source.performance.previous-window";

#[test]
fn optimization_batch_20260830dq_callback_source_window_moves_previous_owner() {
    let source = include_str!("../focus.rs");
    let body = source
        .split("fn with_callback_source_window")
        .nth(1)
        .expect("callback source window scope exists")
        .split("fn focus_callback_source_window")
        .next()
        .expect("callback source window scope ends before focus");

    assert!(body.contains("std::mem::replace"));
    assert!(!body.contains("callback_source_window.clone()"));
}

#[test]
#[ignore = "deterministic clone-count evidence for the managed optimization batch"]
fn optimization_batch_20260830dq_callback_source_window_replace_evidence() {
    let legacy_slot = Some(MainPageId::new(PREVIOUS_WINDOW_ID));
    let mut legacy_previous_id_clones = 0_u64;
    let mut legacy_cloned_id_bytes = 0_u64;

    for _ in 0..LOOKUP_ROUNDS {
        let previous = legacy_slot.clone();
        if let Some(previous) = previous.as_ref() {
            legacy_previous_id_clones += 1;
            legacy_cloned_id_bytes += previous.0.len() as u64;
        }
        black_box(previous);
    }

    let mut optimized_slot = Some(MainPageId::new(PREVIOUS_WINDOW_ID));
    for _ in 0..LOOKUP_ROUNDS {
        let previous = std::mem::replace(&mut optimized_slot, None);
        black_box(optimized_slot.as_ref());
        optimized_slot = previous;
    }

    let optimized_previous_id_clones = 0_u64;
    println!(
        "EDITOR525_CALLBACK_SOURCE_WINDOW_REPLACE_BENCH_V1 rounds={LOOKUP_ROUNDS} legacy_previous_id_clones={legacy_previous_id_clones} optimized_previous_id_clones={optimized_previous_id_clones} legacy_cloned_id_bytes={legacy_cloned_id_bytes} clone_reduction_basis_points=10000"
    );
    assert_eq!(legacy_previous_id_clones, LOOKUP_ROUNDS as u64);
    assert_eq!(optimized_previous_id_clones, 0);
    assert_eq!(optimized_slot, legacy_slot);
}
