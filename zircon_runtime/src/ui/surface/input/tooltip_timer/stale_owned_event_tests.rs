use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ig_runtime_stale_tooltip_moves_owned_event() {
    let tooltip_id = benchmark_tooltip_id(64 * 1024);
    let allocation = tooltip_id.as_ptr();
    let tooltip = tooltip_event(tooltip_id);

    let event = owned_tooltip_timer_event(tooltip);
    let UiInputEvent::TooltipTimer(tooltip) = event else {
        panic!("owned tooltip helper must retain the tooltip event kind");
    };

    assert_eq!(tooltip.tooltip_id.as_ptr(), allocation);
    assert_eq!(tooltip.kind, UiTooltipTimerInputEventKind::Elapsed);
    assert_eq!(tooltip.owner, None);
}

#[test]
fn optimization_batch_20260828ig_runtime_stale_branch_consumes_tooltip_event() {
    let source = include_str!("../tooltip_timer.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_tooltip_timer_input")
        .nth(1)
        .and_then(|body| body.split("fn owned_tooltip_timer_event").next())
        .expect("tooltip timer dispatch implementation");
    let stale_branch = dispatch
        .split("if !tooltip_timer_matches_retained_state(surface, &tooltip)")
        .nth(1)
        .and_then(|body| {
            body.split("let event = UiInputEvent::TooltipTimer(tooltip.clone())")
                .next()
        })
        .expect("stale tooltip branch before the retained-state path");
    let owned_event = source
        .split("fn owned_tooltip_timer_event")
        .nth(1)
        .and_then(|body| body.split("fn with_tooltip_route_policy").next())
        .expect("owned tooltip event helper");

    assert!(stale_branch.contains("owned_tooltip_timer_event(tooltip)"));
    assert!(!stale_branch.contains("tooltip.clone()"));
    assert!(owned_event.contains("UiInputEvent::TooltipTimer(tooltip)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ig_runtime_owned_stale_tooltip_event_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_stale_tooltip_event(tooltip_event(
        benchmark_tooltip_id(64 * 1024),
    )));
    black_box(owned_tooltip_timer_event(tooltip_event(
        benchmark_tooltip_id(64 * 1024),
    )));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_events = benchmark_events(ITERATIONS, 64 * 1024);
        let optimized_events = benchmark_events(ITERATIONS, 64 * 1024);
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_events(legacy_events, legacy_stale_tooltip_event));
            optimized_samples.push(measure_events(optimized_events, owned_tooltip_timer_event));
        } else {
            optimized_samples.push(measure_events(optimized_events, owned_tooltip_timer_event));
            legacy_samples.push(measure_events(legacy_events, legacy_stale_tooltip_event));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME279_OWNED_STALE_TOOLTIP_TIMER_EVENT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn tooltip_event(tooltip_id: String) -> UiTooltipTimerInputEvent {
    UiTooltipTimerInputEvent {
        metadata: Default::default(),
        kind: UiTooltipTimerInputEventKind::Elapsed,
        tooltip_id,
        owner: None,
    }
}

fn benchmark_tooltip_id(bytes: usize) -> String {
    "tooltip-id/".repeat(bytes / 11)
}

fn benchmark_events(count: usize, bytes: usize) -> Vec<UiTooltipTimerInputEvent> {
    (0..count)
        .map(|_| tooltip_event(benchmark_tooltip_id(bytes)))
        .collect()
}

fn legacy_stale_tooltip_event(tooltip: UiTooltipTimerInputEvent) -> UiInputEvent {
    UiInputEvent::TooltipTimer(tooltip.clone())
}

fn measure_events(
    events: Vec<UiTooltipTimerInputEvent>,
    mut convert: impl FnMut(UiTooltipTimerInputEvent) -> UiInputEvent,
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
