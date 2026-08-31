use std::hint::black_box;
use std::sync::{Arc, Barrier};
use std::time::Instant;

use crate::core::framework::events::{
    EngineEvent, EngineEventDeliveryPolicy, EngineEventTryReceiveError, EventBusDiagnosticsMode,
};

use super::{DisconnectedSubscriberIds, EventBus};

const SAMPLE_PAIRS: usize = 31;
const RECORDS_PER_SAMPLE: usize = 200_000;

#[test]
fn optimization_batch_20260829z_runtime299_single_disconnect_cleanup_preserves_behavior() {
    let bus = EventBus::new(EventBusDiagnosticsMode::Disabled);
    let subscription = bus.subscribe(
        "runtime.optimization.single-disconnect",
        EngineEventDeliveryPolicy::Lossless,
    );
    let topic = bus
        .state
        .topic("runtime.optimization.single-disconnect")
        .expect("subscribed topic");
    topic
        .snapshot_subscribers()
        .first()
        .expect("subscribed receiver")
        .deactivate_and_drain();

    bus.publish(EngineEvent {
        topic: "runtime.optimization.single-disconnect".to_string(),
        payload: serde_json::json!({ "sequence": 1 }),
    });

    assert!(
        bus.state
            .topic("runtime.optimization.single-disconnect")
            .is_none()
    );
    assert_eq!(
        subscription.try_recv(),
        Err(EngineEventTryReceiveError::Disconnected)
    );
}

#[test]
fn optimization_batch_20260829z_runtime299_single_disconnect_id_stays_inline() {
    let source = include_str!("../publish.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("pub(super) fn publish")
        .nth(1)
        .expect("event publish implementation");

    assert!(implementation.contains("struct DisconnectedSubscriberIds"));
    assert!(body.contains("DisconnectedSubscriberIds::default()"));
    assert!(implementation.contains("std::slice::from_ref"));
    assert!(!body.contains("get_or_insert_with(Vec::new)"));
}

#[test]
fn optimization_batch_hi_runtime591_empty_topic_checks_subscribers_before_arc_creation() {
    let source = include_str!("../publish.rs");
    let subscriber_check = source
        .find("subscribers.is_empty()")
        .expect("empty subscriber fast path");
    let event_allocation = source
        .find("Arc::new(event)")
        .expect("event Arc allocation");
    assert!(subscriber_check < event_allocation);
}

#[test]
fn optimization_batch_hi_runtime591_pending_subscription_keeps_empty_topic_publish_semantics() {
    let bus = EventBus::new(EventBusDiagnosticsMode::Disabled);
    let topic_name = "runtime.optimization.pending-subscription";
    let reservation_entered = Arc::new(Barrier::new(2));
    let release_reservation = Arc::new(Barrier::new(2));
    let state = Arc::clone(&bus.state);
    let worker_entered = Arc::clone(&reservation_entered);
    let worker_release = Arc::clone(&release_reservation);
    let subscriber = std::thread::spawn(move || {
        state.subscribe_after_reservation_for_test(
            topic_name.to_string(),
            EngineEventDeliveryPolicy::Lossless,
            || {
                worker_entered.wait();
                worker_release.wait();
            },
        )
    });

    reservation_entered.wait();
    bus.publish(EngineEvent {
        topic: topic_name.to_string(),
        payload: serde_json::json!({ "sequence": 1 }),
    });
    release_reservation.wait();
    let subscription = subscriber.join().expect("subscription worker");
    assert_eq!(
        subscription.try_recv(),
        Err(EngineEventTryReceiveError::Empty)
    );

    bus.publish(EngineEvent {
        topic: topic_name.to_string(),
        payload: serde_json::json!({ "sequence": 2 }),
    });
    assert_eq!(
        subscription.recv().expect("post-reservation event").payload,
        serde_json::json!({ "sequence": 2 })
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829z_runtime299_inline_single_disconnect_id_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME299_INLINE_SINGLE_DISCONNECT_ID_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
records_per_sample={RECORDS_PER_SAMPLE} disconnected_ids_per_record=1 \
legacy_result_allocations_per_record=1 optimized_result_allocations_per_record=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_runtime591_empty_topic_payload_arc_bench() {
    const SAMPLE_PAIRS: usize = 17;
    const RECORDS_PER_SAMPLE: usize = 100_000;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_empty_topic(false, RECORDS_PER_SAMPLE));
            optimized_samples.push(measure_empty_topic(true, RECORDS_PER_SAMPLE));
        } else {
            optimized_samples.push(measure_empty_topic(true, RECORDS_PER_SAMPLE));
            legacy_samples.push(measure_empty_topic(false, RECORDS_PER_SAMPLE));
        }
    }
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME591_EMPTY_TOPIC_PAYLOAD_ARC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} records_per_sample={RECORDS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85));
}

fn measure_empty_topic(optimized: bool, records_per_sample: usize) -> u128 {
    let bus = EventBus::new(EventBusDiagnosticsMode::Disabled);
    let topic_name = "runtime.optimization.empty-topic";
    let reservation_entered = Arc::new(Barrier::new(2));
    let release_reservation = Arc::new(Barrier::new(2));
    let state = Arc::clone(&bus.state);
    let worker_entered = Arc::clone(&reservation_entered);
    let worker_release = Arc::clone(&release_reservation);
    let subscriber = std::thread::spawn(move || {
        state.subscribe_after_reservation_for_test(
            topic_name.to_string(),
            EngineEventDeliveryPolicy::Lossless,
            || {
                worker_entered.wait();
                worker_release.wait();
            },
        )
    });
    reservation_entered.wait();
    let events = (0..records_per_sample)
        .map(|sequence| EngineEvent {
            topic: topic_name.to_string(),
            payload: serde_json::json!({ "sequence": sequence }),
        })
        .collect::<Vec<_>>();
    let started = Instant::now();
    for event in events {
        if optimized {
            bus.publish(event);
        } else {
            legacy_publish_without_subscribers(&bus, event);
        }
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    release_reservation.wait();
    black_box(subscriber.join().expect("subscription worker"));
    elapsed
}

fn legacy_publish_without_subscribers(bus: &EventBus, event: EngineEvent) {
    let started = bus.state.diagnostics.record_published_and_capture_time();
    let Some(topic) = bus.state.topic(&event.topic) else {
        bus.state.diagnostics.record_publish_duration(started);
        return;
    };
    let event = Arc::new(event);
    let _delivery = topic.lock_delivery();
    let subscribers = topic.snapshot_subscribers();
    black_box((event, subscribers));
    bus.state.diagnostics.record_publish_duration(started);
}

fn legacy_record_single(disconnected_id: u64) -> usize {
    let mut ids: Option<Vec<u64>> = None;
    ids.get_or_insert_with(Vec::new).push(disconnected_id);
    let ids = black_box(ids);
    ids.as_ref().map(Vec::len).unwrap_or_default()
}

fn optimized_record_single(disconnected_id: u64) -> usize {
    let mut ids = DisconnectedSubscriberIds::default();
    ids.push(disconnected_id);
    black_box(&ids);
    ids.len()
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for id in 0..RECORDS_PER_SAMPLE as u64 {
        checksum = checksum.wrapping_add(if optimized {
            optimized_record_single(black_box(id))
        } else {
            legacy_record_single(black_box(id))
        });
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
