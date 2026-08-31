use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::{dispatch::UiInputEventMetadata, event_ui::UiNodeId};

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ib_runtime_toast_event_reuses_owned_id_allocation() {
    let toast = benchmark_toast(4 * 1024);
    let allocation = toast.toast_id.as_ptr();

    let (target, event) = into_toast_timer_event(toast);
    let UiInputEvent::ToastTimer(toast) = event else {
        panic!("expected toast timer event");
    };

    assert_eq!(target, UiNodeId::new(73));
    assert_eq!(toast.toast_id.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260828ib_runtime_toast_dispatch_moves_event_after_default_action() {
    let source = include_str!("../toast_timer.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_toast_timer_input")
        .nth(1)
        .and_then(|body| body.split("fn into_toast_timer_event").next())
        .expect("toast timer dispatch implementation");

    let default_action = dispatch
        .find("apply_default_toast_timeout_component_event")
        .expect("default toast action");
    let event_move = dispatch
        .find("into_toast_timer_event(toast)")
        .expect("owned toast event move");
    assert!(default_action < event_move);
    assert!(!dispatch.contains("toast.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ib_runtime_owned_toast_timer_event_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_toast_timer_event(benchmark_toast(64 * 1024)));
    black_box(into_toast_timer_event(benchmark_toast(64 * 1024)));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| benchmark_toast(64 * 1024))
            .collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS)
            .map(|_| benchmark_toast(64 * 1024))
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_toasts(legacy_inputs, |toast| {
                black_box(legacy_toast_timer_event(toast));
            }));
            optimized_samples.push(measure_toasts(optimized_inputs, |toast| {
                black_box(into_toast_timer_event(toast));
            }));
        } else {
            optimized_samples.push(measure_toasts(optimized_inputs, |toast| {
                black_box(into_toast_timer_event(toast));
            }));
            legacy_samples.push(measure_toasts(legacy_inputs, |toast| {
                black_box(legacy_toast_timer_event(toast));
            }));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME274_OWNED_TOAST_TIMER_EVENT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_toast(id_bytes: usize) -> UiToastTimerInputEvent {
    UiToastTimerInputEvent {
        metadata: UiInputEventMetadata::default(),
        target: UiNodeId::new(73),
        toast_id: "toast".repeat(id_bytes / 5),
    }
}

fn legacy_toast_timer_event(toast: UiToastTimerInputEvent) -> UiInputEvent {
    UiInputEvent::ToastTimer(toast.clone())
}

fn measure_toasts(
    toasts: Vec<UiToastTimerInputEvent>,
    mut convert: impl FnMut(UiToastTimerInputEvent),
) -> u128 {
    let started = Instant::now();
    for toast in toasts {
        convert(black_box(toast));
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
