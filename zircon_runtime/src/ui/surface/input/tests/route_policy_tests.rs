use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata, UiPointerEvent,
        UiPointerInputEvent, UiPointerSource, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiPoint,
    surface::UiPointerEventKind,
    tree::UiTreeNode,
};

use crate::ui::surface::surface::UiSurface;

use super::{annotate_result_route_policy, annotate_route_policy, preview_tunnel_for_bubble};

#[test]
fn runtime200_generic_route_preview_preserves_bubble_precedence() {
    let bubble_path = vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3)];
    let focus_path = vec![UiNodeId::new(9), UiNodeId::new(10)];
    let preview_source = if bubble_path.is_empty() {
        &focus_path
    } else {
        &bubble_path
    };
    assert_eq!(
        preview_tunnel_for_bubble(preview_source),
        vec![UiNodeId::new(3), UiNodeId::new(2), UiNodeId::new(1)]
    );

    let empty_bubble = Vec::new();
    let preview_source = if empty_bubble.is_empty() {
        &focus_path
    } else {
        &empty_bubble
    };
    assert_eq!(
        preview_tunnel_for_bubble(preview_source),
        vec![UiNodeId::new(10), UiNodeId::new(9)]
    );
}

#[test]
#[ignore = "managed release evidence"]
fn runtime200_generic_route_preview_borrow_release_benchmark() {
    const ROUTE_DEPTH: usize = 512;
    const ITERATIONS: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    const LEGACY_PATH_COPIES: usize = SAMPLE_PAIRS * ITERATIONS;
    const LEGACY_NODE_COPIES: usize = LEGACY_PATH_COPIES * ROUTE_DEPTH;
    let bubble_path = (0..ROUTE_DEPTH)
        .map(|index| UiNodeId::new(index as u64))
        .collect::<Vec<_>>();
    let focus_path = Vec::new();

    let mut legacy = || {
        measure_ns(ITERATIONS, || {
            let route_path = if bubble_path.is_empty() {
                focus_path.clone()
            } else {
                bubble_path.clone()
            };
            black_box(preview_tunnel_for_bubble(&route_path));
        })
    };
    let mut optimized = || {
        measure_ns(ITERATIONS, || {
            let preview_source = if bubble_path.is_empty() {
                &focus_path
            } else {
                &bubble_path
            };
            black_box(preview_tunnel_for_bubble(preview_source));
        })
    };

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(legacy());
            optimized_ns.push(optimized());
        } else {
            optimized_ns.push(optimized());
            legacy_ns.push(legacy());
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "borrowed route preview must reduce P95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "RUNTIME200_ROUTE_PATH_BORROW_BENCH_V1 route_depth={ROUTE_DEPTH} iterations={ITERATIONS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_path_copies={LEGACY_PATH_COPIES} optimized_path_copies=0 legacy_node_copies={LEGACY_NODE_COPIES} optimized_node_copies=0 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn measure_ns(iterations: usize, mut operation: impl FnMut()) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    started.elapsed().as_nanos()
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

fn editable_text_route_surface(target: UiNodeId) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime77.editable_text.route_policy"));
    surface
        .tree
        .insert_root(UiTreeNode::new(target, UiNodePath::new("root/text")));
    surface
}

fn editable_text_route_result(payload: &str, target: UiNodeId) -> UiInputDispatchResult {
    let event = UiInputEvent::Text(UiTextInputEvent {
        metadata: UiInputEventMetadata::default(),
        text: payload.to_owned(),
    });
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
    result.diagnostics.route_target = Some(target);
    result
        .diagnostics
        .notes
        .push("existing diagnostic note".to_owned());
    result
}

#[test]
fn runtime77_editable_text_route_context_borrow_preserves_annotations() {
    let target = UiNodeId::new(42);
    let surface = editable_text_route_surface(target);
    let payload = "multi-kilobyte-text-event-".repeat(4096);
    let initial = editable_text_route_result(&payload, target);
    let mut legacy = initial.clone();
    let mut borrowed = initial;

    let legacy_event = legacy.event.clone();
    annotate_route_policy(&surface, &legacy_event, &mut legacy);
    super::super::route_steps::annotate_result_route_steps(&mut legacy);
    annotate_result_route_policy(&surface, &mut borrowed);
    super::super::route_steps::annotate_result_route_steps(&mut borrowed);

    assert_eq!(legacy.event, borrowed.event);
    assert_eq!(
        legacy.diagnostics.route_policy,
        borrowed.diagnostics.route_policy
    );
    assert_eq!(legacy.diagnostics.notes, borrowed.diagnostics.notes);
    assert_eq!(
        legacy.diagnostics.route_trace,
        borrowed.diagnostics.route_trace
    );
    assert_eq!(
        legacy.diagnostics.route_steps,
        borrowed.diagnostics.route_steps
    );
    assert_eq!(
        borrowed.diagnostics.route_policy,
        zircon_runtime_interface::ui::dispatch::UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        borrowed
            .diagnostics
            .notes
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["existing diagnostic note"]
    );
    assert_eq!(borrowed.diagnostics.route_trace.target, Some(target));
    assert_eq!(borrowed.diagnostics.route_trace.bubble_path, [target]);
    assert_eq!(
        match &borrowed.event {
            UiInputEvent::Text(text) => text.text.as_str(),
            _ => unreachable!("the editable-text route result retains its text event"),
        },
        payload.as_str()
    );
}

#[test]
#[ignore = "managed Release comparison"]
fn runtime77_editable_text_route_context_borrow_release_benchmark() {
    const PAYLOAD_BYTES: usize = 16 * 1024;
    const ITERATIONS: usize = 2_048;
    const SAMPLE_PAIRS: usize = 101;
    let target = UiNodeId::new(42);
    let surface = editable_text_route_surface(target);
    let payload = "x".repeat(PAYLOAD_BYTES);
    let mut legacy_result = editable_text_route_result(&payload, target);
    let mut borrowed_result = editable_text_route_result(&payload, target);

    let mut legacy = || {
        measure_ns(ITERATIONS, || {
            let event = black_box(legacy_result.event.clone());
            annotate_route_policy(&surface, &event, &mut legacy_result);
            black_box(&legacy_result.diagnostics);
        })
    };
    let mut borrowed = || {
        measure_ns(ITERATIONS, || {
            annotate_result_route_policy(&surface, &mut borrowed_result);
            black_box(&borrowed_result.diagnostics);
        })
    };

    for _ in 0..3 {
        legacy();
        borrowed();
    }
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(legacy());
            borrowed_ns.push(borrowed());
        } else {
            borrowed_ns.push(borrowed());
            legacy_ns.push(legacy());
        }
    }
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let borrowed_p95_ns = nearest_rank(&borrowed_ns, 95);
    println!(
        "RUNTIME77_EDITABLE_TEXT_ROUTE_CONTEXT_BORROW_BENCH_V1 payload_bytes={PAYLOAD_BYTES} iterations={ITERATIONS} sample_pairs={SAMPLE_PAIRS} warmup_batches=3 pair_order=alternating_legacy_even legacy_event_clones_per_iteration=1 borrowed_event_clones_per_iteration=0 legacy_event_clone_bytes_lower_bound_per_batch={} borrowed_event_clone_bytes_per_batch=0 legacy_p95_ns={legacy_p95_ns} borrowed_p95_ns={borrowed_p95_ns} legacy_ns={} borrowed_ns={}",
        PAYLOAD_BYTES * ITERATIONS,
        join_samples(&legacy_ns),
        join_samples(&borrowed_ns),
    );
}

#[test]
fn runtime77_borrowed_route_projection_preserves_pointer_notes_and_trace() {
    let target = UiNodeId::new(43);
    let surface = editable_text_route_surface(target);
    let mut metadata = UiInputEventMetadata::default();
    metadata.pointer_source = UiPointerSource::Touch;
    let event = UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(4.0, 5.0)),
        precise_scroll: None,
    });
    let mut legacy = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
    legacy.diagnostics.route_target = Some(target);
    let mut borrowed = legacy.clone();

    let legacy_event = legacy.event.clone();
    annotate_route_policy(&surface, &legacy_event, &mut legacy);
    annotate_result_route_policy(&surface, &mut borrowed);

    assert_eq!(
        legacy.diagnostics.route_policy,
        borrowed.diagnostics.route_policy
    );
    assert_eq!(legacy.diagnostics.notes, borrowed.diagnostics.notes);
    assert_eq!(
        legacy.diagnostics.route_trace,
        borrowed.diagnostics.route_trace
    );
    assert_eq!(
        borrowed.diagnostics.route_policy,
        zircon_runtime_interface::ui::dispatch::UiInputRoutePolicy::Bubble
    );
    assert_eq!(
        borrowed
            .diagnostics
            .notes
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["pointer_source=Touch", "touch_like_pointer"]
    );
    assert_eq!(borrowed.diagnostics.route_trace.target, Some(target));
    assert_eq!(borrowed.diagnostics.route_trace.bubble_path, [target]);
}
