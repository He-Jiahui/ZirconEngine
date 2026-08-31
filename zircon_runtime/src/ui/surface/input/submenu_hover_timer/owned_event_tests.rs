use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::{dispatch::UiInputEventMetadata, event_ui::UiNodeId};

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ic_runtime_submenu_event_reuses_owned_option_id() {
    let submenu_hover = benchmark_submenu_hover(4 * 1024);
    let allocation = submenu_hover.option_id.as_ptr();

    let (target, event) = into_submenu_hover_timer_event(submenu_hover);
    let UiInputEvent::SubmenuHoverTimer(submenu_hover) = event else {
        panic!("expected submenu hover timer event");
    };

    assert_eq!(target, UiNodeId::new(79));
    assert_eq!(submenu_hover.option_id.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260828ic_runtime_submenu_dispatch_moves_event_after_state_reads() {
    let source = include_str!("../submenu_hover_timer.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_submenu_hover_timer_input")
        .nth(1)
        .and_then(|body| body.split("fn into_submenu_hover_timer_event").next())
        .expect("submenu hover timer dispatch implementation");
    let conversion = source
        .split("fn into_submenu_hover_timer_event")
        .nth(1)
        .and_then(|body| body.split("fn with_submenu_hover_route_policy").next())
        .expect("owned submenu hover event conversion");

    let state_read = dispatch
        .find("submenu_hover_delay_ms_for_component_node")
        .expect("submenu retained-state read");
    let event_move = dispatch
        .find("into_submenu_hover_timer_event(submenu_hover)")
        .expect("owned submenu hover event move");
    assert!(state_read < event_move);
    assert!(!dispatch.contains("submenu_hover.clone()"));
    assert!(conversion.contains("UiInputEvent::SubmenuHoverTimer(submenu_hover)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ic_runtime_owned_submenu_hover_event_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_submenu_hover_event(benchmark_submenu_hover(
        64 * 1024,
    )));
    black_box(into_submenu_hover_timer_event(benchmark_submenu_hover(
        64 * 1024,
    )));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| benchmark_submenu_hover(64 * 1024))
            .collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS)
            .map(|_| benchmark_submenu_hover(64 * 1024))
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_submenu_events(legacy_inputs, |event| {
                black_box(legacy_submenu_hover_event(event));
            }));
            optimized_samples.push(measure_submenu_events(optimized_inputs, |event| {
                black_box(into_submenu_hover_timer_event(event));
            }));
        } else {
            optimized_samples.push(measure_submenu_events(optimized_inputs, |event| {
                black_box(into_submenu_hover_timer_event(event));
            }));
            legacy_samples.push(measure_submenu_events(legacy_inputs, |event| {
                black_box(legacy_submenu_hover_event(event));
            }));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME275_OWNED_SUBMENU_HOVER_TIMER_EVENT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_submenu_hover(option_id_bytes: usize) -> UiSubmenuHoverTimerInputEvent {
    UiSubmenuHoverTimerInputEvent {
        metadata: UiInputEventMetadata::default(),
        target: UiNodeId::new(79),
        option_id: "option".repeat(option_id_bytes / 6),
    }
}

fn legacy_submenu_hover_event(submenu_hover: UiSubmenuHoverTimerInputEvent) -> UiInputEvent {
    UiInputEvent::SubmenuHoverTimer(submenu_hover.clone())
}

fn measure_submenu_events(
    events: Vec<UiSubmenuHoverTimerInputEvent>,
    mut convert: impl FnMut(UiSubmenuHoverTimerInputEvent),
) -> u128 {
    let started = Instant::now();
    for event in events {
        convert(black_box(event));
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
