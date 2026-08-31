use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchReply, UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata, UiPopupInputEvent, UiPopupInputEventKind,
    },
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::{UiDirtyFlags, UiTreeNode},
};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn summary_window_diagnostics_skip_string_materialization() {
    let mut summary = window_result();
    mark_optional_window_event_result(
        &mut summary,
        Some("window_focus_gained"),
        UiInputDiagnosticsMode::Summary,
    );
    mark_window_event_result(
        &mut summary,
        "window_normalized_input",
        UiInputDiagnosticsMode::Summary,
    );

    assert!(summary.diagnostics.notes.is_empty());
    assert!(summary.diagnostics.truncation.is_empty());
    assert!(
        handled_window_event_result(window_event(), UiInputDiagnosticsMode::Summary)
            .diagnostics
            .handled_phase
            .is_none()
    );
}

#[test]
fn full_window_diagnostics_preserve_bounded_labels() {
    let mut full = window_result();
    mark_optional_window_event_result(
        &mut full,
        Some("window_focus_gained"),
        UiInputDiagnosticsMode::Full,
    );
    mark_window_event_result(
        &mut full,
        "window_normalized_input",
        UiInputDiagnosticsMode::Full,
    );

    assert_eq!(
        full.diagnostics
            .notes
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "window_focus_gained",
            "window_event",
            "window_input_pump",
            "window_normalized_input",
        ]
    );
    assert!(
        handled_window_event_result(window_event(), UiInputDiagnosticsMode::Full)
            .diagnostics
            .handled_phase
            .is_some()
    );
}

fn window_result() -> UiInputDispatchResult {
    UiInputDispatchResult::new(window_event(), UiDispatchReply::unhandled())
}

fn window_event() -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: UiInputEventMetadata::default(),
        kind: UiPopupInputEventKind::Dismissed,
        popup_id: "window.test".to_string(),
        owner: None,
        anchor: None,
    })
}

#[test]
fn optimization_batch_20260826aq_window_root_iteration_marks_every_root() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.window-roots"));
    for node_id in 1..=4 {
        surface.tree.insert_root(UiTreeNode::new(
            UiNodeId::new(node_id),
            UiNodePath::new(format!("root-{node_id}")),
        ));
    }
    surface.clear_dirty_flags();
    let dirty = UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        ..UiDirtyFlags::default()
    };

    mark_roots_dirty(&mut surface, dirty).expect("all roots remain valid");

    for node_id in 1..=4 {
        assert_eq!(
            surface.tree.node(UiNodeId::new(node_id)).unwrap().dirty,
            dirty
        );
    }
    assert_eq!(surface.invalidation.pending_changed_node_count(), 4);
}

#[test]
fn optimization_batch_20260826aq_window_root_iteration_avoids_root_snapshot() {
    let source = include_str!("../window_pump.rs");
    let mark_roots = bounded_function(source, "fn mark_roots_dirty", "fn layout_metrics_dirty");

    assert!(mark_roots.contains("let root_count = surface.tree.roots.len()"));
    assert!(mark_roots.contains("surface.tree.roots[index]"));
    assert!(!mark_roots.contains("roots.clone()"));
    assert!(!mark_roots.contains("to_vec()"));
    assert!(!mark_roots.contains("collect"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826aq_window_root_borrowed_iteration_p95() {
    const ROOTS: usize = 16_384;
    const PROBES: usize = 32;
    let roots = (1..=ROOTS as u64).map(UiNodeId::new).collect::<Vec<_>>();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(PROBES, || legacy_root_checksum(&roots)));
            optimized_ns.push(measure_ns(PROBES, || borrowed_root_checksum(&roots)));
        } else {
            optimized_ns.push(measure_ns(PROBES, || borrowed_root_checksum(&roots)));
            legacy_ns.push(measure_ns(PROBES, || legacy_root_checksum(&roots)));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "borrowed root iteration P95 must be at least 30% below root snapshots: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME77_WINDOW_ROOT_BORROWED_ITERATION_BENCH_V1 roots={ROOTS} probes_per_sample={PROBES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_root_snapshots_per_sample={PROBES} optimized_root_snapshots_per_sample=0 legacy_root_id_copies_per_sample={} optimized_root_id_copies_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        ROOTS * PROBES,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn legacy_root_checksum(roots: &[UiNodeId]) -> u64 {
    roots
        .to_vec()
        .into_iter()
        .fold(0u64, |checksum, node_id| checksum.wrapping_add(node_id.0))
}

fn borrowed_root_checksum(roots: &[UiNodeId]) -> u64 {
    let mut checksum = 0u64;
    for index in 0..roots.len() {
        checksum = checksum.wrapping_add(roots[index].0);
    }
    checksum
}

fn measure_ns(probes: usize, operation: impl Fn() -> u64) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..probes {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_function<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("function start")
        .split(end)
        .next()
        .expect("function end")
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
