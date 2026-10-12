use crate::core::framework::events::*;
use crate::core::runtime::EventBus;
use std::sync::{Arc, Barrier};

#[test]
fn optimization_batch_20260829z_runtime299_single_disconnect_cleanup_preserves_behavior() {
    let bus = EventBus::default();
    let retired = bus
        .subscribe(
            "runtime.cleanup",
            EngineEventDeliveryPolicy::Reliable {
                limits: EventRetentionLimits::default(),
            },
        )
        .unwrap();
    let live = bus
        .subscribe(
            "runtime.cleanup",
            EngineEventDeliveryPolicy::Reliable {
                limits: EventRetentionLimits::default(),
            },
        )
        .unwrap();
    drop(retired);
    let receipt = bus
        .try_publish(EngineEvent {
            topic: "runtime.cleanup".into(),
            payload: serde_json::json!({"sequence":1}),
        })
        .unwrap();
    assert_eq!(receipt.subscribers, 1);
    assert_eq!(
        live.recv().unwrap().decode_payload().unwrap()["sequence"],
        1
    );
}

#[test]
fn optimization_batch_hi_runtime591_pending_subscription_keeps_empty_topic_publish_semantics() {
    let bus = EventBus::default();
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let state = Arc::clone(&bus.state);
    let worker_entered = entered.clone();
    let worker_release = release.clone();
    let worker = std::thread::spawn(move || {
        state
            .subscribe_after_reservation_for_test(
                "runtime.pending".into(),
                EngineEventDeliveryPolicy::Reliable {
                    limits: EventRetentionLimits::default(),
                },
                || {
                    worker_entered.wait();
                    worker_release.wait();
                },
            )
            .unwrap()
    });
    entered.wait();
    let rejected = bus
        .try_publish(EngineEvent {
            topic: "runtime.pending".into(),
            payload: serde_json::json!({"sequence":1}),
        })
        .unwrap_err();
    assert_eq!(rejected.reason, EngineEventPublishRejection::NoSubscribers);
    assert_eq!(bus.retention_snapshot().retained_bytes, 0);
    release.wait();
    let subscribed = worker.join().unwrap();
    assert!(matches!(
        subscribed.try_recv(),
        Err(EngineEventTryReceiveError::Empty)
    ));
    bus.try_publish(rejected.event).unwrap();
    assert_eq!(
        subscribed.recv().unwrap().decode_payload().unwrap()["sequence"],
        1
    );
}
