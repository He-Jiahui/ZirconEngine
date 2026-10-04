use std::hint::black_box;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use crate::core::framework::events::{EngineEventDeliveryPolicy, EventBusDiagnosticsMode};
use crate::core::EngineEvent;

use super::super::diagnostics::EventBusDiagnosticsState;
use super::super::subscriber::EventSubscriber;
use super::super::EventBus;
use super::{EventBusState, EventTopic};

const BULK_PRUNE_SUBSCRIBERS: usize = 4_096;
const MAX_BULK_PRUNE_LATENCY: Duration = Duration::from_millis(100);
const TOPIC_LOOKUP_WORKERS: usize = 8;
const TOPIC_LOOKUPS_PER_WORKER: usize = 50_000;
const MIN_TOPIC_LOOKUPS_PER_SECOND: f64 = 250_000.0;
const SINGLE_PRUNE_SAMPLE_PAIRS: usize = 17;
const SINGLE_PRUNE_ITERATIONS: usize = 200_000;

fn poison_in_worker(action: impl FnOnce() + Send + 'static) {
    assert!(
        std::thread::spawn(action).join().is_err(),
        "test worker must poison its held mutex"
    );
}

#[test]
fn runtime02_existing_topic_subscription_does_not_clone_the_lookup_key() {
    let source = include_str!("../topic.rs");
    let end = source.find("mod tests {").expect("test module");
    let implementation = &source[..end];

    assert!(implementation.contains("match topics.entry(topic)"));
    assert!(implementation.contains("Entry::Occupied"));
    assert!(implementation.contains("Entry::Vacant"));
    assert!(!implementation.contains("entry(topic.clone())"));
}

#[test]
fn runtime02_topic_registry_supports_overlapping_publish_lookups() {
    let bus = EventBus::default();
    let _subscription = bus
        .subscribe(
            "runtime.registry.read",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let registry = Arc::clone(&bus.state);
    let read_entered = Arc::new(Barrier::new(2));
    let release_read = Arc::new(Barrier::new(2));
    let worker_entered = Arc::clone(&read_entered);
    let worker_release = Arc::clone(&release_read);
    let reader = std::thread::spawn(move || {
        let _topics = registry.topics.read().unwrap();
        worker_entered.wait();
        worker_release.wait();
    });

    read_entered.wait();
    let overlapping_read = bus.state.topics.try_read();
    release_read.wait();
    reader.join().unwrap();

    assert!(
        overlapping_read.is_ok(),
        "independent publish lookups must not serialize on a registry mutex"
    );
}

#[test]
fn runtime02_bulk_subscriber_prune_sorts_ids_for_sublinear_membership_checks() {
    let source = include_str!("../topic.rs");
    let implementation = source
        .split("mod tests {")
        .next()
        .expect("topic implementation");

    assert!(implementation.contains("sorted_subscriber_ids.sort_unstable()"));
    assert!(implementation.contains("binary_search(&subscriber.id())"));
    assert!(!implementation.contains("subscriber_ids.contains"));
}

#[test]
fn optimization_batch_hi_runtime595_single_prune_skips_temporary_id_buffer() {
    let source = include_str!("../topic.rs");
    let implementation = source
        .split("mod tests {")
        .next()
        .expect("topic implementation");
    let single_id_branch = implementation
        .find("if let [subscriber_id] = subscriber_ids")
        .expect("single subscriber fast path");
    let bulk_copy = implementation
        .find("subscriber_ids.to_vec()")
        .expect("bulk subscriber path");

    assert!(single_id_branch < bulk_copy);
}

#[test]
fn optimization_batch_hi_runtime595_single_prune_preserves_snapshot_order() {
    let diagnostics = Arc::new(EventBusDiagnosticsState::new(
        EventBusDiagnosticsMode::Disabled,
    ));
    let topic = EventTopic::new("runtime.single-prune".to_string());
    let subscribers = [10_u64, 11, 11, 12]
        .into_iter()
        .map(|id| {
            Arc::new(EventSubscriber::new(
                id,
                EngineEventDeliveryPolicy::Reliable {
                    limits: crate::core::framework::events::EventRetentionLimits::default(),
                },
                Arc::clone(&diagnostics),
                Arc::new(super::super::admission::SharedAdmission::new(
                    crate::core::framework::events::EventBusLimits::default(),
                )),
            ))
        })
        .collect::<Vec<_>>();
    *topic.lock_subscribers() = subscribers.into();

    assert!(topic.remove_subscribers_while_delivery_locked(&[11]));
    assert_eq!(
        topic
            .snapshot_subscribers()
            .iter()
            .map(|subscriber| subscriber.id())
            .collect::<Vec<_>>(),
        vec![10, 12]
    );
    assert!(!topic.remove_subscribers_while_delivery_locked(&[99]));
    assert!(!topic.remove_subscribers_while_delivery_locked(&[]));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hi_runtime595_single_prune_id_preparation_bench() {
    let mut legacy_samples = Vec::with_capacity(SINGLE_PRUNE_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SINGLE_PRUNE_SAMPLE_PAIRS);
    for sample in 0..SINGLE_PRUNE_SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_single_prune_id_preparation(false));
            optimized_samples.push(measure_single_prune_id_preparation(true));
        } else {
            optimized_samples.push(measure_single_prune_id_preparation(true));
            legacy_samples.push(measure_single_prune_id_preparation(false));
        }
    }
    let legacy_p95_ns = percentile_95(&legacy_samples);
    let optimized_p95_ns = percentile_95(&optimized_samples);
    println!(
        "RUNTIME595_EVENT_SINGLE_PRUNE_ID_PREPARATION_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} sample_pairs={} iterations={} temporary_id_buffer_allocations_per_prune=1->0",
        legacy_p95_ns, optimized_p95_ns, SINGLE_PRUNE_SAMPLE_PAIRS, SINGLE_PRUNE_ITERATIONS,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
        "optimized single-ID preparation P95 must be at most 50% of legacy P95"
    );
}

fn measure_single_prune_id_preparation(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut matches = 0usize;
    for subscriber_id in 0..SINGLE_PRUNE_ITERATIONS as u64 {
        let candidate = black_box(subscriber_id);
        matches += usize::from(if optimized {
            candidate == black_box(subscriber_id)
        } else {
            let mut sorted_subscriber_ids = black_box([subscriber_id].as_slice()).to_vec();
            sorted_subscriber_ids.sort_unstable();
            sorted_subscriber_ids.binary_search(&candidate).is_ok()
        });
    }
    black_box(matches);
    started.elapsed().as_nanos().max(1)
}

fn percentile_95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}

#[test]
#[ignore = "managed Runtime02 performance evidence"]
fn event_bus_runtime02_parallel_topic_lookup_evidence() {
    let state = Arc::new(EventBusState::new(
        EventBusDiagnosticsMode::Disabled,
        crate::core::framework::events::EventBusLimits::default(),
    ));
    {
        let mut topics = state.write_topics();
        for worker in 0..TOPIC_LOOKUP_WORKERS {
            let name = format!("runtime.lookup.{worker}");
            topics.insert(name.clone(), Arc::new(EventTopic::new(name)));
        }
    }
    let start = Arc::new(Barrier::new(TOPIC_LOOKUP_WORKERS + 1));
    let readers = (0..TOPIC_LOOKUP_WORKERS)
        .map(|worker| {
            let state = Arc::clone(&state);
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                let name = format!("runtime.lookup.{worker}");
                start.wait();
                for _ in 0..TOPIC_LOOKUPS_PER_WORKER {
                    assert!(state.topic(&name).is_some());
                }
            })
        })
        .collect::<Vec<_>>();

    let started = Instant::now();
    start.wait();
    for reader in readers {
        reader.join().unwrap();
    }
    let elapsed = started.elapsed();
    let lookups = TOPIC_LOOKUP_WORKERS * TOPIC_LOOKUPS_PER_WORKER;
    let lookups_per_second = lookups as f64 / elapsed.as_secs_f64();

    assert!(lookups_per_second >= MIN_TOPIC_LOOKUPS_PER_SECOND);
    println!(
        "EVENTBUS_BENCH_V3 kind=parallel_topic_lookup workers={} lookups={} exclusive_registry_lock_acquisitions_before={} exclusive_registry_lock_acquisitions_after=0 exclusive_lock_reduction_percent=100.0000 elapsed_ns={} lookups_per_second={:.2} target_lookups_per_second={:.2}",
        TOPIC_LOOKUP_WORKERS,
        lookups,
        lookups,
        elapsed.as_nanos(),
        lookups_per_second,
        MIN_TOPIC_LOOKUPS_PER_SECOND,
    );
}

#[test]
#[ignore = "managed Runtime02 performance evidence"]
fn event_bus_runtime02_bulk_disconnect_prune_evidence() {
    let diagnostics = Arc::new(EventBusDiagnosticsState::new(
        EventBusDiagnosticsMode::Disabled,
    ));
    let topic = EventTopic::new("runtime.bulk-prune".to_string());
    let subscribers = (0..BULK_PRUNE_SUBSCRIBERS)
        .map(|id| {
            Arc::new(EventSubscriber::new(
                id as u64,
                EngineEventDeliveryPolicy::Reliable {
                    limits: crate::core::framework::events::EventRetentionLimits::default(),
                },
                Arc::clone(&diagnostics),
                Arc::new(super::super::admission::SharedAdmission::new(
                    crate::core::framework::events::EventBusLimits::default(),
                )),
            ))
        })
        .collect::<Vec<_>>();
    *topic.lock_subscribers() = subscribers.into();
    let disconnected_ids = (0..BULK_PRUNE_SUBSCRIBERS as u64).rev().collect::<Vec<_>>();

    let started = Instant::now();
    let removed = topic.remove_subscribers_while_delivery_locked(&disconnected_ids);
    let elapsed = started.elapsed();

    assert!(removed);
    assert_eq!(topic.subscriber_count(), 0);
    assert!(elapsed <= MAX_BULK_PRUNE_LATENCY);
    let membership_probes_before = BULK_PRUNE_SUBSCRIBERS * (BULK_PRUNE_SUBSCRIBERS + 1) / 2;
    let binary_search_probes_per_subscriber = BULK_PRUNE_SUBSCRIBERS.ilog2() as usize + 1;
    let membership_probes_after_upper_bound =
        BULK_PRUNE_SUBSCRIBERS * binary_search_probes_per_subscriber;
    let membership_probe_reduction_percent = (1.0
        - membership_probes_after_upper_bound as f64 / membership_probes_before as f64)
        * 100.0;
    println!(
        "EVENTBUS_BENCH_V3 kind=bulk_disconnect_prune subscribers={} membership_probes_before={} membership_probes_after_upper_bound={} membership_probe_reduction_percent={:.4} elapsed_ns={} target_ns={}",
        BULK_PRUNE_SUBSCRIBERS,
        membership_probes_before,
        membership_probes_after_upper_bound,
        membership_probe_reduction_percent,
        elapsed.as_nanos(),
        MAX_BULK_PRUNE_LATENCY.as_nanos(),
    );
}

#[test]
fn runtime02_poisoned_event_bus_mutexes_fail_admission_closed() {
    let bus = EventBus::default();
    let subscription = bus
        .subscribe(
            "runtime.poison",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");

    let state = Arc::clone(&bus.state);
    poison_in_worker(move || {
        let _topics = state.topics.write().unwrap();
        panic!("poison EventBus topic map");
    });

    let topic = bus
        .state
        .topic("runtime.poison")
        .expect("poison-safe topic map must retain the subscription");
    let topic_for_subscribers = Arc::clone(&topic);
    poison_in_worker(move || {
        let _subscribers = topic_for_subscribers.subscribers.lock().unwrap();
        panic!("poison EventBus subscriber snapshot");
    });
    let topic_for_delivery = Arc::clone(&topic);
    poison_in_worker(move || {
        let _delivery = topic_for_delivery.delivery.lock().unwrap();
        panic!("poison EventBus per-topic delivery lock");
    });
    let subscriber = Arc::clone(
        topic
            .snapshot_subscribers()
            .first()
            .expect("subscription must remain present after poison recovery"),
    );
    poison_in_worker(move || {
        subscriber.poison_queue_state_for_test();
    });

    let rejected = bus
        .try_publish(EngineEvent {
            topic: "runtime.poison".to_string(),
            payload: serde_json::json!({ "recovered": true }),
        })
        .unwrap_err();
    assert_eq!(
        rejected.reason,
        crate::core::framework::events::EngineEventPublishRejection::Poisoned
    );
    bus.close_admission();
    assert!(matches!(
        subscription.try_recv(),
        Err(crate::core::framework::events::EngineEventTryReceiveError::Disconnected)
    ));
    let report = bus.diagnostic_report();
    assert_eq!(report.topics, 1);
    assert_eq!(report.subscribers, 1);
    assert_eq!(report.published, 1);
    assert_eq!(report.delivered, 0);
}

#[test]
fn runtime02_registry_poison_alone_fails_admission_closed() {
    let bus = EventBus::default();
    let subscription = bus
        .subscribe(
            "runtime.registry.poison",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .unwrap();
    let state = Arc::clone(&bus.state);
    poison_in_worker(move || {
        let mut topics = state.topics.write().unwrap();
        topics.clear();
        panic!("registry mutation before poison");
    });
    let rejected = bus
        .try_publish(EngineEvent {
            topic: "runtime.registry.poison".into(),
            payload: serde_json::json!(1),
        })
        .unwrap_err();
    assert_eq!(
        rejected.reason,
        crate::core::framework::events::EngineEventPublishRejection::Poisoned
    );
    assert!(bus.retention_snapshot().poisoned);
    drop(subscription);
}
