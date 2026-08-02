use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EngineEventReceiveError, EngineEventReceiveTimeoutError,
    EngineEventTryReceiveError, EventBusDiagnosticsMode,
};
use crate::core::{EngineEvent, EventBus};

use super::super::super::*;

#[test]
fn event_bus_and_config_store_roundtrip() {
    let runtime = CoreRuntime::new();
    let events = runtime
        .handle()
        .subscribe_events("editor.selection", EngineEventDeliveryPolicy::Lossless);
    runtime.publish_event("editor.selection", serde_json::json!({ "node": 7 }));
    let event = events.recv().unwrap();
    assert_eq!(event.payload["node"], 7);

    runtime
        .handle()
        .store_config("editor.theme", &serde_json::json!({ "name": "TokyoNight" }))
        .unwrap();
    let theme: Value = runtime.load_config("editor.theme").unwrap();
    assert_eq!(theme["name"], "TokyoNight");
}

#[test]
fn event_bus_prunes_closed_subscribers_after_snapshot_publish() {
    let bus = EventBus::default();
    let closed_events = bus.subscribe("runtime.tick", EngineEventDeliveryPolicy::Lossless);
    let live_events = bus.subscribe("runtime.tick", EngineEventDeliveryPolicy::Lossless);
    drop(closed_events);

    bus.publish(EngineEvent {
        topic: "runtime.tick".to_string(),
        payload: serde_json::json!({ "frame": 1 }),
    });
    let event = live_events.recv().unwrap();
    assert_eq!(event.payload["frame"], 1);

    bus.publish(EngineEvent {
        topic: "runtime.tick".to_string(),
        payload: serde_json::json!({ "frame": 2 }),
    });
    let event = live_events.recv().unwrap();
    assert_eq!(event.payload["frame"], 2);
}

#[test]
fn event_bus_bounded_drop_oldest_caps_paused_subscriber_and_reports_drop() {
    let bus = EventBus::default();
    let events = bus.subscribe(
        "runtime.tick",
        EngineEventDeliveryPolicy::BoundedDropOldest {
            capacity: NonZeroUsize::new(2).unwrap(),
        },
    );

    for frame in 1..=3 {
        bus.publish(EngineEvent {
            topic: "runtime.tick".to_string(),
            payload: serde_json::json!({ "frame": frame }),
        });
    }

    assert_eq!(events.recv().unwrap().payload["frame"], 2);
    assert_eq!(events.recv().unwrap().payload["frame"], 3);
    assert_eq!(events.try_recv(), Err(EngineEventTryReceiveError::Empty));
    let report = bus.diagnostic_report();
    assert_eq!(report.published, 3);
    assert_eq!(report.delivered, 3);
    assert_eq!(report.dropped, 1);
    assert_eq!(report.queued, 0);
    assert_eq!(report.peak_queued, 2);
}

#[test]
fn event_bus_fanout_shares_one_payload_for_1_2_5_100_subscribers() {
    for subscriber_count in [1, 2, 5, 100] {
        let bus = EventBus::default();
        let subscriptions = (0..subscriber_count)
            .map(|_| bus.subscribe("runtime.snapshot", EngineEventDeliveryPolicy::Lossless))
            .collect::<Vec<_>>();

        bus.publish(EngineEvent {
            topic: "runtime.snapshot".to_string(),
            payload: serde_json::json!({ "nodes": [1, 2, 3, 4], "blob": "x".repeat(4096) }),
        });

        let first = subscriptions[0].recv().unwrap();
        for subscription in subscriptions.iter().skip(1) {
            let event = subscription.recv().unwrap();
            assert!(Arc::ptr_eq(&first, &event));
        }
        assert_eq!(bus.diagnostic_report().delivered, subscriber_count as u64);
    }
}

#[test]
fn event_bus_latest_policy_coalesces_to_the_newest_event() {
    let bus = EventBus::default();
    let events = bus.subscribe("runtime.cursor", EngineEventDeliveryPolicy::Latest);

    for sample in 1..=64 {
        bus.publish(EngineEvent {
            topic: "runtime.cursor".to_string(),
            payload: serde_json::json!({ "sample": sample }),
        });
    }

    assert_eq!(events.recv().unwrap().payload["sample"], 64);
    assert_eq!(events.try_recv(), Err(EngineEventTryReceiveError::Empty));
    assert_eq!(bus.diagnostic_report().dropped, 63);
}

#[test]
fn event_bus_capacity_one_peak_never_exceeds_the_physical_queue_capacity() {
    let bus = Arc::new(EventBus::default());
    let events = bus.subscribe("runtime.capacity", EngineEventDeliveryPolicy::Latest);
    let publishing = Arc::new(AtomicBool::new(true));
    let consumer_publishing = Arc::clone(&publishing);
    let consumer = std::thread::spawn(move || {
        while consumer_publishing.load(Ordering::Acquire) {
            match events.try_recv() {
                Ok(_) | Err(EngineEventTryReceiveError::Empty) => std::thread::yield_now(),
                Err(EngineEventTryReceiveError::Disconnected) => break,
            }
        }
        while events.try_recv().is_ok() {}
    });

    for sequence in 0..4_096 {
        bus.publish(EngineEvent {
            topic: "runtime.capacity".to_string(),
            payload: serde_json::json!({ "sequence": sequence }),
        });
    }
    publishing.store(false, Ordering::Release);
    consumer.join().unwrap();

    let report = bus.diagnostic_report();
    assert_eq!(report.queued, 0);
    assert!(report.peak_queued <= 1);
}

#[test]
fn event_bus_lossless_policy_preserves_same_topic_publish_order() {
    let bus = EventBus::default();
    let events = bus.subscribe("runtime.sequence", EngineEventDeliveryPolicy::Lossless);

    for sequence in 0..256 {
        bus.publish(EngineEvent {
            topic: "runtime.sequence".to_string(),
            payload: serde_json::json!({ "sequence": sequence }),
        });
    }

    for expected in 0..256 {
        assert_eq!(events.recv().unwrap().payload["sequence"], expected);
    }
    assert_eq!(bus.diagnostic_report().dropped, 0);
}

#[test]
fn event_bus_reports_queue_age_when_a_paused_consumer_resumes() {
    let bus = EventBus::default();
    let events = bus.subscribe("runtime.age", EngineEventDeliveryPolicy::Lossless);
    bus.publish(EngineEvent {
        topic: "runtime.age".to_string(),
        payload: Value::Null,
    });

    std::thread::sleep(Duration::from_millis(5));
    events.recv().unwrap();

    let report = bus.diagnostic_report();
    assert_eq!(report.queue_age_samples, 1);
    assert!(report.total_queue_age_ms >= 1.0);
    assert!(report.max_queue_age_ms >= 1.0);
    assert_eq!(report.queued, 0);
}

#[test]
fn event_subscription_disconnects_when_the_last_event_bus_owner_drops() {
    let bus = EventBus::default();
    let polling = bus.subscribe("runtime.shutdown", EngineEventDeliveryPolicy::Lossless);
    let blocking = bus.subscribe("runtime.shutdown", EngineEventDeliveryPolicy::Lossless);
    let ready = Arc::new(Barrier::new(2));
    let waiter_ready = Arc::clone(&ready);
    let waiter = std::thread::spawn(move || {
        waiter_ready.wait();
        blocking.recv()
    });

    ready.wait();
    let deadline = Instant::now() + Duration::from_secs(1);
    while bus.diagnostic_report().waiting_receivers != 1 {
        assert!(
            Instant::now() < deadline,
            "blocking receiver did not enter the condition-variable wait"
        );
        std::thread::yield_now();
    }
    drop(bus);

    assert_eq!(
        polling.try_recv(),
        Err(EngineEventTryReceiveError::Disconnected)
    );
    assert_eq!(
        polling.recv_timeout(Duration::from_millis(10)),
        Err(EngineEventReceiveTimeoutError::Disconnected)
    );
    assert_eq!(
        waiter.join().expect("blocking event waiter should exit"),
        Err(EngineEventReceiveError::Disconnected)
    );
}

#[test]
fn event_subscription_overflowing_timeout_waits_until_an_event_arrives() {
    let bus = EventBus::default();
    let events = bus.subscribe("runtime.long_wait", EngineEventDeliveryPolicy::Lossless);
    let waiter = std::thread::spawn(move || events.recv_timeout(Duration::MAX));
    let deadline = Instant::now() + Duration::from_secs(1);
    while bus.diagnostic_report().waiting_receivers != 1 {
        assert!(
            Instant::now() < deadline,
            "overflowing timeout receiver returned instead of waiting"
        );
        std::thread::yield_now();
    }

    bus.publish(EngineEvent {
        topic: "runtime.long_wait".to_string(),
        payload: serde_json::json!({ "arrived": true }),
    });
    assert_eq!(
        waiter
            .join()
            .expect("long-timeout waiter should exit")
            .expect("long-timeout waiter should receive the event")
            .payload["arrived"],
        true
    );
}

#[test]
fn core_runtime_exposes_its_live_event_bus_diagnostics() {
    let runtime = CoreRuntime::new();
    let events = runtime.subscribe_events("runtime.metrics", EngineEventDeliveryPolicy::Lossless);
    runtime.publish_event("runtime.metrics", serde_json::json!({ "frame": 1 }));

    let queued = runtime.event_bus_diagnostics();
    assert_eq!(queued.topics, 1);
    assert_eq!(queued.subscribers, 1);
    assert_eq!(queued.published, 1);
    assert_eq!(queued.delivered, 1);
    assert_eq!(queued.queued, 1);

    events.recv().unwrap();
    assert_eq!(runtime.event_bus_diagnostics().queued, 0);
}

#[test]
fn event_bus_disabled_diagnostics_skip_timing_and_counter_collection() {
    let bus = EventBus::new(EventBusDiagnosticsMode::Disabled);
    let events = bus.subscribe("runtime.silent", EngineEventDeliveryPolicy::Lossless);
    bus.publish(EngineEvent {
        topic: "runtime.silent".to_string(),
        payload: Value::Null,
    });
    events.recv().unwrap();

    let report = bus.diagnostic_report();
    assert!(!report.enabled);
    assert_eq!(report.topics, 1);
    assert_eq!(report.subscribers, 1);
    assert_eq!(report.published, 0);
    assert_eq!(report.delivered, 0);
    assert_eq!(report.queued, 0);
    assert_eq!(report.queue_age_samples, 0);
    assert_eq!(report.publish_samples, 0);
    assert_eq!(report.delivery_lock_wait_samples, 0);
}

#[test]
fn event_bus_uncontended_publish_does_not_report_delivery_lock_wait() {
    let bus = EventBus::default();
    let events = bus.subscribe("runtime.uncontended", EngineEventDeliveryPolicy::Lossless);

    bus.publish(EngineEvent {
        topic: "runtime.uncontended".to_string(),
        payload: Value::Null,
    });
    events.recv().unwrap();

    let report = bus.diagnostic_report();
    assert_eq!(report.waiting_publishers, 0);
    assert_eq!(report.delivery_lock_wait_samples, 0);
    assert_eq!(report.total_delivery_lock_wait_ms, 0.0);
    assert_eq!(report.max_delivery_lock_wait_ms, 0.0);
}

#[test]
fn event_bus_reports_same_topic_publisher_delivery_lock_wait() {
    let bus = Arc::new(EventBus::default());
    let events = bus.subscribe("runtime.contended", EngineEventDeliveryPolicy::Lossless);
    let lock_entered = Arc::new(Barrier::new(2));
    let release_lock = Arc::new(Barrier::new(2));
    let holder_bus = Arc::clone(&bus);
    let holder_entered = Arc::clone(&lock_entered);
    let holder_release = Arc::clone(&release_lock);
    let holder = std::thread::spawn(move || {
        holder_bus.hold_topic_delivery_for_test("runtime.contended", || {
            holder_entered.wait();
            holder_release.wait();
        });
    });
    lock_entered.wait();

    let publisher_bus = Arc::clone(&bus);
    let publisher = std::thread::spawn(move || {
        publisher_bus.publish(EngineEvent {
            topic: "runtime.contended".to_string(),
            payload: Value::Null,
        });
    });
    let deadline = Instant::now() + Duration::from_secs(1);
    while bus.diagnostic_report().waiting_publishers != 1 {
        assert!(
            Instant::now() < deadline,
            "publisher did not enter the delivery-lock wait"
        );
        std::thread::yield_now();
    }
    std::thread::sleep(Duration::from_millis(2));
    release_lock.wait();
    holder.join().unwrap();
    publisher.join().unwrap();
    events.recv().unwrap();

    let report = bus.diagnostic_report();
    assert_eq!(report.waiting_publishers, 0);
    assert_eq!(report.delivery_lock_wait_samples, 1);
    assert!(report.total_delivery_lock_wait_ms >= 1.0);
    assert!(report.max_delivery_lock_wait_ms >= 1.0);
}

#[test]
fn event_bus_concurrent_same_topic_publishers_share_one_exact_fanout_interleaving() {
    const PUBLISHER_COUNT: usize = 4;
    const EVENTS_PER_PUBLISHER: usize = 128;

    let bus = Arc::new(EventBus::default());
    let first_events = bus.subscribe("runtime.concurrent", EngineEventDeliveryPolicy::Lossless);
    let second_events = bus.subscribe("runtime.concurrent", EngineEventDeliveryPolicy::Lossless);
    let start = Arc::new(Barrier::new(PUBLISHER_COUNT + 1));
    let publishers = (0..PUBLISHER_COUNT)
        .map(|producer| {
            let bus = Arc::clone(&bus);
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                start.wait();
                for sequence in 0..EVENTS_PER_PUBLISHER {
                    bus.publish(EngineEvent {
                        topic: "runtime.concurrent".to_string(),
                        payload: serde_json::json!({
                            "producer": producer,
                            "sequence": sequence,
                        }),
                    });
                }
            })
        })
        .collect::<Vec<_>>();

    start.wait();
    for publisher in publishers {
        publisher.join().unwrap();
    }
    let receive_all = |events: &dyn crate::core::framework::events::EngineEventSubscription| {
        (0..PUBLISHER_COUNT * EVENTS_PER_PUBLISHER)
            .map(|_| {
                let event = events
                    .recv_timeout(Duration::from_secs(5))
                    .expect("concurrent publisher event should arrive");
                (
                    event.payload["producer"].as_u64().unwrap(),
                    event.payload["sequence"].as_u64().unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    let first_received = receive_all(first_events.as_ref());
    let second_received = receive_all(second_events.as_ref());
    assert_eq!(first_received, second_received);

    let mut next_sequence = [0_u64; PUBLISHER_COUNT];
    for (producer, sequence) in first_received {
        let producer = producer as usize;
        assert_eq!(sequence, next_sequence[producer]);
        next_sequence[producer] += 1;
    }

    assert_eq!(
        next_sequence,
        [EVENTS_PER_PUBLISHER as u64; PUBLISHER_COUNT]
    );
    let report = bus.diagnostic_report();
    assert_eq!(
        report.published,
        (PUBLISHER_COUNT * EVENTS_PER_PUBLISHER) as u64
    );
    assert_eq!(report.delivered, report.published * 2);
    assert_eq!(report.queued, 0);
    assert!(report.peak_queued >= 1);
}

#[test]
fn event_bus_allows_another_topic_to_progress_while_subscribe_waits_on_delivery() {
    let bus = Arc::new(EventBus::default());
    let blocked_events = bus.subscribe("runtime.blocked", EngineEventDeliveryPolicy::Lossless);
    let free_events = bus.subscribe("runtime.free", EngineEventDeliveryPolicy::Lossless);
    let lock_entered = Arc::new(Barrier::new(2));
    let release_lock = Arc::new(Barrier::new(2));
    let reservation_reached = Arc::new(Barrier::new(2));
    let holder_bus = Arc::clone(&bus);
    let holder_entered = Arc::clone(&lock_entered);
    let holder_release = Arc::clone(&release_lock);
    let holder = std::thread::spawn(move || {
        holder_bus.hold_topic_delivery_for_test("runtime.blocked", || {
            holder_entered.wait();
            holder_release.wait();
        });
    });

    lock_entered.wait();
    let subscribe_bus = Arc::clone(&bus);
    let subscribe_reserved = Arc::clone(&reservation_reached);
    let subscriber = std::thread::spawn(move || {
        subscribe_bus.subscribe_after_reservation_for_test(
            "runtime.blocked",
            EngineEventDeliveryPolicy::Lossless,
            || {
                subscribe_reserved.wait();
            },
        )
    });
    reservation_reached.wait();

    let (progress_sender, progress_receiver) = mpsc::channel();
    let free_bus = Arc::clone(&bus);
    let free_publisher = std::thread::spawn(move || {
        free_bus.publish(EngineEvent {
            topic: "runtime.free".to_string(),
            payload: serde_json::json!({ "progress": true }),
        });
        progress_sender.send(()).unwrap();
    });
    let progress = progress_receiver.recv_timeout(Duration::from_secs(1));
    release_lock.wait();
    holder.join().unwrap();
    let added_events = subscriber.join().unwrap();
    free_publisher.join().unwrap();
    progress.expect("unrelated topic must progress while subscribe waits on another topic");
    assert_eq!(free_events.recv().unwrap().payload["progress"], true);

    bus.publish(EngineEvent {
        topic: "runtime.blocked".to_string(),
        payload: serde_json::json!({ "released": true }),
    });
    assert_eq!(blocked_events.recv().unwrap().payload["released"], true);
    assert_eq!(added_events.recv().unwrap().payload["released"], true);
}

#[test]
fn event_bus_reservation_prevents_last_drop_from_orphaning_a_new_subscription() {
    let bus = Arc::new(EventBus::default());
    let anchor = bus.subscribe("runtime.race", EngineEventDeliveryPolicy::Lossless);
    let reserved = Arc::new(Barrier::new(2));
    let continue_subscription = Arc::new(Barrier::new(2));
    let subscribe_bus = Arc::clone(&bus);
    let subscribe_reserved = Arc::clone(&reserved);
    let subscribe_continue = Arc::clone(&continue_subscription);
    let subscriber = std::thread::spawn(move || {
        subscribe_bus.subscribe_after_reservation_for_test(
            "runtime.race",
            EngineEventDeliveryPolicy::Lossless,
            || {
                subscribe_reserved.wait();
                subscribe_continue.wait();
            },
        )
    });

    reserved.wait();
    drop(anchor);
    assert_eq!(bus.diagnostic_report().topics, 1);
    assert_eq!(bus.diagnostic_report().subscribers, 0);
    continue_subscription.wait();

    let events = subscriber.join().unwrap();
    bus.publish(EngineEvent {
        topic: "runtime.race".to_string(),
        payload: serde_json::json!({ "iteration": 1 }),
    });
    assert_eq!(
        events
            .recv_timeout(Duration::from_secs(1))
            .expect("reserved subscription must remain registered")
            .payload["iteration"],
        1
    );
}
