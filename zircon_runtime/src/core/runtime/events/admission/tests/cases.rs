use super::*;
use crate::core::framework::events::*;
use crate::core::runtime::EventBus;
use std::num::NonZeroUsize;

fn budget(events: usize, bytes: usize) -> EventRetentionLimits {
    EventRetentionLimits {
        max_events: NonZeroUsize::new(events).unwrap(),
        max_bytes: NonZeroUsize::new(bytes).unwrap(),
    }
}
fn event(n: usize) -> EngineEvent {
    EngineEvent {
        topic: "admission.test".into(),
        payload: serde_json::json!({"n": n}),
    }
}
fn reliable(events: usize, bytes: usize) -> EngineEventDeliveryPolicy {
    EngineEventDeliveryPolicy::Reliable {
        limits: budget(events, bytes),
    }
}

#[test]
fn rejected_fanout_returns_owned_event_without_admitting_an_unfilled_peer() {
    let bus = EventBus::default();
    let full = bus.subscribe("admission.test", reliable(1, 4096)).unwrap();
    bus.try_publish(event(1)).unwrap();
    let peer = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
    let expected = event(2);
    let rejected = bus.try_publish(expected.clone()).unwrap_err();
    assert_eq!(rejected.event, expected);
    assert!(matches!(
        rejected.reason,
        EngineEventPublishRejection::Backpressured { .. }
    ));
    assert!(matches!(
        peer.try_recv(),
        Err(EngineEventTryReceiveError::Empty)
    ));
    assert_eq!(full.recv().unwrap().decode_payload().unwrap()["n"], 1);
}

#[test]
fn received_clones_stay_charged_after_unsubscribe_until_the_last_clone_drops() {
    let bus = EventBus::default();
    let sub = bus.subscribe("admission.test", reliable(1, 4096)).unwrap();
    bus.try_publish(event(1)).unwrap();
    let received = sub.recv().unwrap();
    let copy = received.clone();
    assert_eq!(bus.retention_snapshot().retained_events, 1);
    assert!(bus.try_publish(event(2)).is_err());
    drop(sub);
    drop(received);
    assert_eq!(bus.retention_snapshot().retained_events, 1);
    drop(copy);
    assert_eq!(bus.retention_snapshot().retained_events, 0);
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
}

#[test]
fn exact_frozen_buffers_do_not_retain_spare_producer_string_capacity() {
    let bus = EventBus::default();
    let sub = bus.subscribe("admission.test", reliable(2, 4096)).unwrap();
    let mut huge = String::with_capacity(1024 * 1024);
    huge.push('x');
    bus.try_publish(EngineEvent {
        topic: "admission.test".into(),
        payload: serde_json::Value::String(huge),
    })
    .unwrap();
    let received = sub.recv().unwrap();
    assert_eq!(received.retained_bytes(), "admission.test".len() + 3);
    assert_eq!(
        bus.retention_snapshot().retained_bytes,
        received.retained_bytes()
    );
    assert_eq!(received.payload_bytes(), br#""x""#);
}

#[test]
fn byte_pressure_is_independent_of_event_count_and_preserves_queue_on_oversize() {
    let bus = EventBus::default();
    let bytes = "admission.test".len() + 7;
    let sub = bus
        .subscribe("admission.test", reliable(10, bytes))
        .unwrap();
    bus.try_publish(event(1)).unwrap();
    assert!(bus.try_publish(event(2)).is_err());
    assert_eq!(sub.recv().unwrap().decode_payload().unwrap()["n"], 1);
    let lossy = bus
        .subscribe(
            "admission.test",
            EngineEventDeliveryPolicy::DropOldest {
                limits: budget(10, bytes),
            },
        )
        .unwrap();
    drop(sub);
    bus.try_publish(event(3)).unwrap();
    let too_large = EngineEvent {
        topic: "admission.test".into(),
        payload: serde_json::json!({"blob":"x".repeat(100)}),
    };
    assert!(bus.try_publish(too_large).is_err());
    assert_eq!(lossy.recv().unwrap().decode_payload().unwrap()["n"], 3);
}

#[test]
fn latest_replaces_only_queued_deliveries_and_reports_every_displacement() {
    let bus = EventBus::default();
    let sub = bus
        .subscribe(
            "admission.test",
            EngineEventDeliveryPolicy::Latest {
                max_retained_bytes: NonZeroUsize::new(4096).unwrap(),
            },
        )
        .unwrap();
    bus.try_publish(event(1)).unwrap();
    let receipt = bus.try_publish(event(2)).unwrap();
    assert_eq!(receipt.replaced_events, 1);
    assert_eq!(receipt.subscribers, 1);
    let held = sub.recv().unwrap();
    let rejected = bus.try_publish(event(3)).unwrap_err();
    drop(held);
    bus.try_publish(rejected.event).unwrap();
    assert_eq!(sub.recv().unwrap().decode_payload().unwrap()["n"], 3);
}

#[test]
fn no_subscriber_and_closed_rejections_preserve_the_input_and_close_is_idempotent() {
    let bus = EventBus::default();
    let input = event(1);
    let rejected = bus.try_publish(input.clone()).unwrap_err();
    assert_eq!(rejected.reason, EngineEventPublishRejection::NoSubscribers);
    assert_eq!(rejected.event, input);
    let sub = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
    bus.try_publish(event(2)).unwrap();
    let first = bus.close_admission();
    assert_eq!(first.drained_events, 1);
    assert_eq!(bus.close_admission().drained_events, 0);
    assert!(matches!(
        sub.try_recv(),
        Err(EngineEventTryReceiveError::Disconnected)
    ));
    assert_eq!(
        bus.try_publish(event(3)).unwrap_err().reason,
        EngineEventPublishRejection::Closed
    );
    assert!(bus.subscribe("admission.test", reliable(4, 4096)).is_err());
    assert_eq!(bus.retention_snapshot().retained_events, 0);
}

#[test]
fn mandatory_global_and_topic_limits_work_with_disabled_diagnostics() {
    for topic_limit in [false, true] {
        let mut limits = EventBusLimits::default();
        if topic_limit {
            limits.topic = budget(1, 4096);
        } else {
            limits.global = budget(1, 4096);
        }
        let bus = EventBus::with_limits(EventBusDiagnosticsMode::Disabled, limits);
        let sub = bus.subscribe("admission.test", reliable(10, 4096)).unwrap();
        bus.try_publish(event(1)).unwrap();
        assert!(bus.try_publish(event(2)).is_err());
        assert_eq!(bus.retention_snapshot().retained_events, 1);
        assert_eq!(bus.diagnostic_report().published, 0);
        drop(sub);
        assert_eq!(bus.retention_snapshot().retained_events, 0);
    }
}

#[test]
fn frozen_payload_is_shared_across_fanout_while_each_retention_lease_is_charged() {
    let bus = EventBus::default();
    let a = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
    let b = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
    bus.try_publish(event(1)).unwrap();
    let av = a.recv().unwrap();
    let bv = b.recv().unwrap();
    assert!(av.shares_payload_with(&bv));
    assert_eq!(bus.retention_snapshot().retained_events, 2);
    assert_eq!(
        bus.retention_snapshot().retained_bytes,
        av.retained_bytes() * 2
    );
    drop(av);
    assert_eq!(bus.retention_snapshot().retained_events, 1);
    drop(bv);
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
}

#[test]
fn drop_oldest_can_displace_multiple_queued_events_for_one_larger_payload() {
    let bus = EventBus::default();
    let small_bytes = "admission.test".len() + 7;
    let sub = bus
        .subscribe(
            "admission.test",
            EngineEventDeliveryPolicy::DropOldest {
                limits: budget(10, small_bytes * 3),
            },
        )
        .unwrap();
    for n in 0..3 {
        bus.try_publish(event(n)).unwrap();
    }
    let receipt = bus
        .try_publish(EngineEvent {
            topic: "admission.test".into(),
            payload: serde_json::Value::String("x".repeat(small_bytes)),
        })
        .unwrap();
    assert_eq!(receipt.replaced_events, 2);
    assert_eq!(receipt.replaced_bytes, small_bytes * 2);
    assert!(bus.retention_snapshot().retained_bytes <= small_bytes * 3);
    assert_eq!(sub.recv().unwrap().decode_payload().unwrap()["n"], 2);
    assert_eq!(
        sub.recv()
            .unwrap()
            .decode_payload()
            .unwrap()
            .as_str()
            .unwrap()
            .len(),
        small_bytes
    );
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
}

#[test]
fn received_handle_survives_close_and_its_clone_releases_bytes_once() {
    let bus = EventBus::default();
    let sub = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
    bus.try_publish(event(1)).unwrap();
    let retained = sub.recv().unwrap();
    let clone = retained.clone();
    bus.close_admission();
    assert_eq!(bus.retention_snapshot().retained_events, 1);
    assert_eq!(retained.decode_payload().unwrap()["n"], 1);
    drop(retained);
    assert_eq!(bus.retention_snapshot().retained_events, 1);
    drop(clone);
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
}

#[test]
fn registry_limits_and_payload_limit_return_typed_rejections() {
    let mut limits = EventBusLimits::default();
    limits.max_subscribers = NonZeroUsize::new(1).unwrap();
    limits.max_topics = NonZeroUsize::new(1).unwrap();
    limits.max_event_payload_bytes = NonZeroUsize::new(7).unwrap();
    let bus = EventBus::with_limits(EventBusDiagnosticsMode::Disabled, limits);
    let sub = bus.subscribe("admission.test", reliable(10, 4096)).unwrap();
    assert!(matches!(
        bus.subscribe("second", reliable(10, 4096)),
        Err(EngineEventSubscribeError::RegistryFull)
    ));
    assert_eq!(
        bus.try_publish(EngineEvent {
            topic: "admission.test".into(),
            payload: serde_json::Value::String("x".repeat(8))
        })
        .unwrap_err()
        .reason,
        EngineEventPublishRejection::PayloadTooLarge
    );
    assert!(matches!(
        sub.try_recv(),
        Err(EngineEventTryReceiveError::Empty)
    ));
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
}

#[test]
fn poisoned_after_queue_mutation_fails_closed_without_new_admission() {
    let bus = EventBus::default();
    let sub = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
    bus.try_publish(event(1)).unwrap();
    let subscriber = bus
        .state
        .topic("admission.test")
        .unwrap()
        .snapshot_subscribers()[0]
        .clone();
    let worker = std::thread::spawn(move || {
        let mut queue = subscriber.lock_queue_state();
        queue.active = false;
        panic!("interrupt queue transaction after mutation");
    });
    assert!(worker.join().is_err());
    assert_eq!(
        bus.try_publish(event(2)).unwrap_err().reason,
        EngineEventPublishRejection::Poisoned
    );
    let close = bus.close_admission();
    assert_eq!(close.retention.retained_events, 0);
    assert_eq!(close.retention.retained_bytes, 0);
    assert!(close.retention.poisoned);
    assert!(matches!(
        sub.try_recv(),
        Err(EngineEventTryReceiveError::Disconnected)
    ));
}

#[test]
fn close_race_preserves_successful_or_owned_rejected_publication_and_wakes_receivers() {
    for _ in 0..32 {
        let bus = EventBus::default();
        let sub = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let publisher = bus.clone();
        let started = barrier.clone();
        let work = std::thread::spawn(move || {
            started.wait();
            publisher.try_publish(event(1))
        });
        barrier.wait();
        bus.close_admission();
        match work.join().unwrap() {
            Ok(receipt) => assert_eq!(receipt.subscribers, 1),
            Err(rejected) => {
                assert_eq!(rejected.event, event(1));
                assert!(matches!(
                    rejected.reason,
                    EngineEventPublishRejection::Closed
                        | EngineEventPublishRejection::NoSubscribers
                ));
            }
        }
        assert!(matches!(
            sub.recv_timeout(std::time::Duration::from_millis(10)),
            Err(EngineEventReceiveTimeoutError::Disconnected)
        ));
        assert_eq!(bus.retention_snapshot().retained_events, 0);
    }
}

#[test]
fn checked_retention_arithmetic_rejects_overflow_without_saturation() {
    assert!(Retained {
        events: usize::MAX,
        bytes: 0
    }
    .add(1, 0)
    .is_none());
    assert!(Retained {
        events: 0,
        bytes: usize::MAX
    }
    .add(0, 1)
    .is_none());
    assert!(Retained::default().subtract(1, 0).is_none());
    assert!(Retained::default().subtract(0, 1).is_none());
}

#[test]
fn close_wakes_a_receiver_already_blocked_on_the_condition_variable() {
    let bus = EventBus::new(EventBusDiagnosticsMode::Enabled);
    let subscription = bus.subscribe("admission.test", reliable(4, 4096)).unwrap();
    let (finished, result) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        finished.send(subscription.recv()).unwrap();
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while bus.diagnostic_report().waiting_receivers != 1 {
        assert!(
            std::time::Instant::now() < deadline,
            "receiver did not begin waiting"
        );
        std::thread::yield_now();
    }
    bus.close_admission();
    assert!(matches!(
        result
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        Err(EngineEventReceiveError::Disconnected)
    ));
    worker.join().unwrap();
    assert_eq!(bus.diagnostic_report().waiting_receivers, 0);
}

#[test]
fn global_and_topic_byte_pressure_reject_all_peers_without_eviction() {
    let bytes = "admission.test".len() + 7;
    for topic_scope in [false, true] {
        let mut limits = EventBusLimits::default();
        if topic_scope {
            limits.topic = budget(100, bytes * 2);
        } else {
            limits.global = budget(100, bytes * 2);
        }
        let bus = EventBus::with_limits(EventBusDiagnosticsMode::Disabled, limits);
        let a = bus
            .subscribe("admission.test", reliable(100, 4096))
            .unwrap();
        let b = bus
            .subscribe("admission.test", reliable(100, 4096))
            .unwrap();
        bus.try_publish(event(1)).unwrap();
        let rejected = bus.try_publish(event(2)).unwrap_err();
        assert_eq!(
            rejected.reason,
            EngineEventPublishRejection::Backpressured {
                scope: if topic_scope {
                    EventBudgetScope::Topic
                } else {
                    EventBudgetScope::Global
                },
            }
        );
        assert_eq!(bus.retention_snapshot().retained_events, 2);
        assert_eq!(bus.retention_snapshot().retained_bytes, bytes * 2);
        assert_eq!(a.recv().unwrap().decode_payload().unwrap()["n"], 1);
        assert_eq!(b.recv().unwrap().decode_payload().unwrap()["n"], 1);
        assert_eq!(bus.retention_snapshot().retained_bytes, 0);
        bus.try_publish(rejected.event).unwrap();
    }
}
