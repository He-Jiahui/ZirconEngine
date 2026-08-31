use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ih_runtime_stale_popup_moves_owned_event() {
    let popup_id = benchmark_popup_id(64 * 1024);
    let allocation = popup_id.as_ptr();
    let popup = popup_event(popup_id);

    let event = owned_popup_input_event(popup);
    let UiInputEvent::Popup(popup) = event else {
        panic!("owned popup helper must retain the popup event kind");
    };

    assert_eq!(popup.popup_id.as_ptr(), allocation);
    assert_eq!(popup.kind, UiPopupInputEventKind::Dismissed);
    assert_eq!(popup.owner, None);
    assert_eq!(popup.anchor, None);
}

#[test]
fn optimization_batch_20260828ih_runtime_stale_branch_consumes_popup_event() {
    let source = include_str!("../popup.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_popup_input")
        .nth(1)
        .and_then(|body| body.split("fn owned_popup_input_event").next())
        .expect("popup dispatch implementation");
    let stale_branch = dispatch
        .split("if !popup_matches_retained_state(surface, &popup)")
        .nth(1)
        .and_then(|body| {
            body.split("let event = UiInputEvent::Popup(popup.clone())")
                .next()
        })
        .expect("stale popup branch before the retained-state path");
    let owned_event = source
        .split("fn owned_popup_input_event")
        .nth(1)
        .and_then(|body| body.split("fn with_popup_route_policy").next())
        .expect("owned popup event helper");

    assert!(stale_branch.contains("owned_popup_input_event(popup)"));
    assert!(!stale_branch.contains("popup.clone()"));
    assert!(owned_event.contains("UiInputEvent::Popup(popup)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ih_runtime_owned_stale_popup_event_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_stale_popup_event(popup_event(benchmark_popup_id(
        64 * 1024,
    ))));
    black_box(owned_popup_input_event(popup_event(benchmark_popup_id(
        64 * 1024,
    ))));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_events = benchmark_events(ITERATIONS, 64 * 1024);
        let optimized_events = benchmark_events(ITERATIONS, 64 * 1024);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_events(legacy_events, legacy_stale_popup_event));
            optimized_samples.push(measure_events(optimized_events, owned_popup_input_event));
        } else {
            optimized_samples.push(measure_events(optimized_events, owned_popup_input_event));
            legacy_samples.push(measure_events(legacy_events, legacy_stale_popup_event));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME280_OWNED_STALE_POPUP_EVENT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn popup_event(popup_id: String) -> UiPopupInputEvent {
    UiPopupInputEvent {
        metadata: Default::default(),
        kind: UiPopupInputEventKind::Dismissed,
        popup_id,
        owner: None,
        anchor: None,
    }
}

fn benchmark_popup_id(bytes: usize) -> String {
    "popup-id/".repeat(bytes / 9)
}

fn benchmark_events(count: usize, bytes: usize) -> Vec<UiPopupInputEvent> {
    (0..count)
        .map(|_| popup_event(benchmark_popup_id(bytes)))
        .collect()
}

fn legacy_stale_popup_event(popup: UiPopupInputEvent) -> UiInputEvent {
    UiInputEvent::Popup(popup.clone())
}

fn measure_events(
    events: Vec<UiPopupInputEvent>,
    mut convert: impl FnMut(UiPopupInputEvent) -> UiInputEvent,
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
