use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::{
    component::{UiDragPayload, UiDragPayloadKind},
    dispatch::{UiSurfaceId, UiWindowId},
};

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ii_runtime_stale_drag_drop_moves_owned_payload() {
    let payload_reference = benchmark_payload_reference(64 * 1024);
    let allocation = payload_reference.as_ptr();
    let drag_drop = drag_drop_event(payload_reference);

    let event = owned_drag_drop_input_event(drag_drop);
    let UiInputEvent::DragDrop(drag_drop) = event else {
        panic!("owned drag-drop helper must retain the drag-drop event kind");
    };
    let payload = drag_drop.payload.expect("drag-drop payload");

    assert_eq!(payload.reference.as_ptr(), allocation);
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert_eq!(drag_drop.kind, UiDragDropInputEventKind::End);
}

#[test]
fn optimization_batch_20260828ii_runtime_stale_branch_consumes_drag_drop_event() {
    let source = include_str!("../drag_drop.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_drag_drop_input")
        .nth(1)
        .and_then(|body| body.split("fn owned_drag_drop_input_event").next())
        .expect("drag-drop dispatch implementation");
    let stale_branch = dispatch
        .split("if !drag_drop_matches_retained_state(surface, &drag_drop, pointer_id)")
        .nth(1)
        .and_then(|body| {
            body.split("let event = UiInputEvent::DragDrop(drag_drop.clone())")
                .next()
        })
        .expect("stale drag-drop branch before the retained-state path");
    let owned_event = source
        .split("fn owned_drag_drop_input_event")
        .nth(1)
        .and_then(|body| body.split("fn with_drag_drop_route_policy").next())
        .expect("owned drag-drop event helper");

    assert!(stale_branch.contains("owned_drag_drop_input_event(drag_drop)"));
    assert!(!stale_branch.contains("drag_drop.clone()"));
    assert!(owned_event.contains("UiInputEvent::DragDrop(drag_drop)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ii_runtime_owned_stale_drag_drop_event_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_stale_drag_drop_event(drag_drop_event(
        benchmark_payload_reference(64 * 1024),
    )));
    black_box(owned_drag_drop_input_event(drag_drop_event(
        benchmark_payload_reference(64 * 1024),
    )));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_events = benchmark_events(ITERATIONS, 64 * 1024);
        let optimized_events = benchmark_events(ITERATIONS, 64 * 1024);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_events(legacy_events, legacy_stale_drag_drop_event));
            optimized_samples.push(measure_events(
                optimized_events,
                owned_drag_drop_input_event,
            ));
        } else {
            optimized_samples.push(measure_events(
                optimized_events,
                owned_drag_drop_input_event,
            ));
            legacy_samples.push(measure_events(legacy_events, legacy_stale_drag_drop_event));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME281_OWNED_STALE_DRAG_DROP_EVENT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn drag_drop_event(payload_reference: String) -> UiDragDropInputEvent {
    let mut metadata = zircon_runtime_interface::ui::dispatch::UiInputEventMetadata::default();
    metadata.window_id = Some(UiWindowId::new(payload_reference.clone()));
    metadata.surface_id = Some(UiSurfaceId::new(payload_reference.clone()));
    UiDragDropInputEvent {
        metadata,
        kind: UiDragDropInputEventKind::End,
        session_id: None,
        point: UiPoint::new(8.0, 13.0),
        payload: Some(Arc::new(UiDragPayload::new(
            UiDragPayloadKind::Asset,
            payload_reference,
        ))),
    }
}

fn benchmark_payload_reference(bytes: usize) -> String {
    "drag-payload/".repeat(bytes / 13)
}

fn benchmark_events(count: usize, bytes: usize) -> Vec<UiDragDropInputEvent> {
    (0..count)
        .map(|_| drag_drop_event(benchmark_payload_reference(bytes)))
        .collect()
}

fn legacy_stale_drag_drop_event(drag_drop: UiDragDropInputEvent) -> UiInputEvent {
    UiInputEvent::DragDrop(drag_drop.clone())
}

fn measure_events(
    events: Vec<UiDragDropInputEvent>,
    mut convert: impl FnMut(UiDragDropInputEvent) -> UiInputEvent,
) -> u128 {
    let started = Instant::now();
    for event in events {
        black_box(convert(black_box(event)));
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
