use std::hint::black_box;
use std::time::Instant;

use crossbeam_channel::{unbounded, Receiver, Sender};
use serde_json::Value;
use zircon_runtime_interface::ui::event_ui::UiInvocationResult;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hx_runtime_single_subscriber_receives_owned_notification() {
    let mut manager = UiEventManager::default();
    let (_, receiver) = manager.subscribe();
    let notification = benchmark_notification(4 * 1024);
    let allocation = notification_payload(&notification).as_ptr();

    manager.broadcast(notification);

    let received = receiver.recv().expect("broadcast notification");
    assert_eq!(notification_payload(&received).as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260828hx_runtime_fanout_clones_only_before_owned_final_delivery() {
    let mut manager = UiEventManager::default();
    let (_, first_receiver) = manager.subscribe();
    let (_, final_receiver) = manager.subscribe();
    let notification = benchmark_notification(4 * 1024);
    let allocation = notification_payload(&notification).as_ptr();

    manager.broadcast(notification);

    let first = first_receiver.recv().expect("first broadcast notification");
    let final_delivery = final_receiver.recv().expect("final broadcast notification");
    assert_eq!(first, final_delivery);
    assert_ne!(notification_payload(&first).as_ptr(), allocation);
    assert_eq!(notification_payload(&final_delivery).as_ptr(), allocation);
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hx_runtime_owned_notification_fanout_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 128;
    let notification = benchmark_notification(64 * 1024);
    let (legacy_sender, legacy_receiver) = unbounded();
    let mut manager = UiEventManager::default();
    let (_, optimized_receiver) = manager.subscribe();

    let _ = legacy_sender.send(notification.clone());
    black_box(legacy_receiver.recv().unwrap());
    manager.broadcast(notification.clone());
    black_box(optimized_receiver.recv().unwrap());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| notification.clone())
            .collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS)
            .map(|_| notification.clone())
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy_fanout(
                &legacy_sender,
                &legacy_receiver,
                legacy_inputs,
            ));
            optimized_samples.push(measure_owned_fanout(
                &manager,
                &optimized_receiver,
                optimized_inputs,
            ));
        } else {
            optimized_samples.push(measure_owned_fanout(
                &manager,
                &optimized_receiver,
                optimized_inputs,
            ));
            legacy_samples.push(measure_legacy_fanout(
                &legacy_sender,
                &legacy_receiver,
                legacy_inputs,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME270_OWNED_NOTIFICATION_FANOUT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_notification(payload_bytes: usize) -> UiNotification {
    UiNotification::Invocation(UiInvocationResult {
        route_id: None,
        binding: None,
        value: Some(Value::String("x".repeat(payload_bytes))),
        error: None,
    })
}

fn notification_payload(notification: &UiNotification) -> &str {
    let UiNotification::Invocation(result) = notification else {
        panic!("expected invocation notification");
    };
    result
        .value
        .as_ref()
        .and_then(Value::as_str)
        .expect("string notification payload")
}

fn measure_legacy_fanout(
    sender: &Sender<UiNotification>,
    receiver: &Receiver<UiNotification>,
    inputs: Vec<UiNotification>,
) -> u128 {
    let started = Instant::now();
    for notification in inputs {
        sender.send(black_box(notification.clone())).unwrap();
        black_box(receiver.recv().unwrap());
    }
    started.elapsed().as_nanos()
}

fn measure_owned_fanout(
    manager: &UiEventManager,
    receiver: &Receiver<UiNotification>,
    inputs: Vec<UiNotification>,
) -> u128 {
    let started = Instant::now();
    for notification in inputs {
        manager.broadcast(black_box(notification));
        black_box(receiver.recv().unwrap());
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
