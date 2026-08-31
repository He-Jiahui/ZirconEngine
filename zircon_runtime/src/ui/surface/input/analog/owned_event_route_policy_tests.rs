use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::dispatch::{UiInputEventMetadata, UiSurfaceId, UiWindowId};

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hw_runtime_moves_owned_analog_event_allocation() {
    let mut event = benchmark_event(4 * 1024);
    let control = analog_event(&event).control.as_ptr();
    let window_id = analog_event(&event)
        .metadata
        .window_id
        .as_ref()
        .expect("window id")
        .0
        .as_ptr();

    let moved = take_owned_input_event(&mut event);

    assert_eq!(analog_event(&moved).control.as_ptr(), control);
    assert_eq!(
        analog_event(&moved)
            .metadata
            .window_id
            .as_ref()
            .expect("moved window id")
            .0
            .as_ptr(),
        window_id
    );
    assert!(analog_event(&event).control.is_empty());
}

#[test]
fn optimization_batch_20260828hw_runtime_route_policy_consumes_and_restores_event() {
    let source = include_str!("../analog.rs");
    let route_policy = source
        .split("fn with_analog_route_policy")
        .nth(1)
        .and_then(|body| body.split("fn analog_with_retained_control_value").next())
        .expect("analog route policy implementation");

    assert!(route_policy.contains("take_owned_input_event(&mut result.event)"));
    assert!(route_policy.contains("result.event = event;"));
    assert!(!route_policy.contains("result.event.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hw_runtime_owned_analog_event_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 128;
    let event = benchmark_event(64 * 1024);

    black_box(event.clone());
    let mut warmup = event.clone();
    black_box(take_owned_input_event(&mut warmup));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let mut optimized_inputs = (0..ITERATIONS).map(|_| event.clone()).collect::<Vec<_>>();
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(<UiInputEvent as Clone>::clone(black_box(&event)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for input in &mut optimized_inputs {
                black_box(take_owned_input_event(black_box(input)));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME269_OWNED_ANALOG_ROUTE_POLICY_EVENT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_event(payload_bytes: usize) -> UiInputEvent {
    let payload = "x".repeat(payload_bytes);
    UiInputEvent::Analog(UiAnalogInputEvent {
        metadata: UiInputEventMetadata {
            window_id: Some(UiWindowId::new(format!("window-{payload}"))),
            surface_id: Some(UiSurfaceId::new(format!("surface-{payload}"))),
            ..UiInputEventMetadata::default()
        },
        control: format!("gamepad.axis.left_x-{payload}"),
        value: 0.75,
    })
}

fn analog_event(event: &UiInputEvent) -> &UiAnalogInputEvent {
    let UiInputEvent::Analog(event) = event else {
        panic!("expected analog input event");
    };
    event
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
