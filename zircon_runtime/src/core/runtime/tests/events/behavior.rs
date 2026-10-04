use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EngineEventReceiveError, EngineEventReceiveTimeoutError,
    EngineEventTryReceiveError, EventBusDiagnosticsMode, DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL,
};
use crate::core::{EngineEvent, EventBus};

use super::super::super::*;

// 通过 CoreRuntime 的门面和独立 EventBus 同时检验调用者可见的投递、生命周期与诊断契约。
// 受控并发用例额外固定同主题全序和不同主题之间的进展边界。

#[test]
fn event_bus_and_config_store_roundtrip() {
    let runtime = CoreRuntime::new();
    let events = runtime
        .handle()
        .subscribe_events(
            "editor.selection",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    runtime
        .try_publish_event("editor.selection", serde_json::json!({ "node": 7 }))
        .expect("event admission");
    let event = events.recv().unwrap();
    assert_eq!(event.decode_payload().unwrap()["node"], 7);

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
    let closed_events = bus
        .subscribe(
            "runtime.tick",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let live_events = bus
        .subscribe(
            "runtime.tick",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    drop(closed_events);

    bus.try_publish(EngineEvent {
        topic: "runtime.tick".to_string(),
        payload: serde_json::json!({ "frame": 1 }),
    })
    .expect("event admission");
    let event = live_events.recv().unwrap();
    assert_eq!(event.decode_payload().unwrap()["frame"], 1);

    bus.try_publish(EngineEvent {
        topic: "runtime.tick".to_string(),
        payload: serde_json::json!({ "frame": 2 }),
    })
    .expect("event admission");
    let event = live_events.recv().unwrap();
    assert_eq!(event.decode_payload().unwrap()["frame"], 2);
}

#[test]
fn event_bus_bounded_drop_oldest_caps_paused_subscriber_and_reports_drop() {
    let bus = EventBus::default();
    let events = bus
        .subscribe(
            "runtime.tick",
            EngineEventDeliveryPolicy::DropOldest {
                limits: crate::core::framework::events::EventRetentionLimits {
                    max_events: NonZeroUsize::new(2).unwrap(),
                    max_bytes: NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
                },
            },
        )
        .expect("bounded subscription");

    for frame in 1..=3 {
        bus.try_publish(EngineEvent {
            topic: "runtime.tick".to_string(),
            payload: serde_json::json!({ "frame": frame }),
        })
        .expect("event admission");
    }

    assert_eq!(events.recv().unwrap().decode_payload().unwrap()["frame"], 2);
    assert_eq!(events.recv().unwrap().decode_payload().unwrap()["frame"], 3);
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
            .map(|_| {
                bus.subscribe(
                    "runtime.snapshot",
                    EngineEventDeliveryPolicy::Reliable {
                        limits: crate::core::framework::events::EventRetentionLimits::default(),
                    },
                )
                .expect("bounded subscription")
            })
            .collect::<Vec<_>>();

        bus.try_publish(EngineEvent {
            topic: "runtime.snapshot".to_string(),
            payload: serde_json::json!({ "nodes": [1, 2, 3, 4], "blob": "x".repeat(4096) }),
        })
        .expect("event admission");

        let first = subscriptions[0].recv().unwrap();
        for subscription in subscriptions.iter().skip(1) {
            let event = subscription.recv().unwrap();
            assert!(first.shares_payload_with(&event));
        }
        assert_eq!(bus.diagnostic_report().delivered, subscriber_count as u64);
    }
}

#[test]
fn event_bus_latest_policy_coalesces_to_the_newest_event() {
    let bus = EventBus::default();
    let events = bus
        .subscribe(
            "runtime.cursor",
            EngineEventDeliveryPolicy::Latest {
                max_retained_bytes: NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
            },
        )
        .expect("bounded subscription");

    for sample in 1..=64 {
        bus.try_publish(EngineEvent {
            topic: "runtime.cursor".to_string(),
            payload: serde_json::json!({ "sample": sample }),
        })
        .expect("event admission");
    }

    assert_eq!(
        events.recv().unwrap().decode_payload().unwrap()["sample"],
        64
    );
    assert_eq!(events.try_recv(), Err(EngineEventTryReceiveError::Empty));
    assert_eq!(bus.diagnostic_report().dropped, 63);
}

// 峰值诊断代表物理队列占用上界；并发消费可暴露计数与入队顺序脱节的瞬时超限。
#[test]
fn event_bus_capacity_one_peak_never_exceeds_the_physical_queue_capacity() {
    let bus = Arc::new(EventBus::default());
    let events = bus
        .subscribe(
            "runtime.capacity",
            EngineEventDeliveryPolicy::Latest {
                max_retained_bytes: NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
            },
        )
        .expect("bounded subscription");
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
        let mut event = EngineEvent {
            topic: "runtime.capacity".into(),
            payload: serde_json::json!({ "sequence": sequence }),
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match bus.try_publish(event) {
                Ok(_) => break,
                Err(rejected) => {
                    assert!(matches!(rejected.reason, crate::core::framework::events::EngineEventPublishRejection::Backpressured { .. }));
                    assert!(
                        Instant::now() < deadline,
                        "retained consumer handle did not release budget"
                    );
                    event = rejected.event;
                    std::thread::yield_now();
                }
            }
        }
    }
    publishing.store(false, Ordering::Release);
    consumer.join().unwrap();

    let report = bus.diagnostic_report();
    assert_eq!(report.queued, 0);
    assert!(report.peak_queued <= 1);
}

#[test]
fn event_bus_reliable_policy_preserves_same_topic_publish_order() {
    let bus = EventBus::default();
    let events = bus
        .subscribe(
            "runtime.sequence",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");

    for sequence in 0..256 {
        bus.try_publish(EngineEvent {
            topic: "runtime.sequence".to_string(),
            payload: serde_json::json!({ "sequence": sequence }),
        })
        .expect("event admission");
    }

    for expected in 0..256 {
        assert_eq!(
            events.recv().unwrap().decode_payload().unwrap()["sequence"],
            expected
        );
    }
    assert_eq!(bus.diagnostic_report().dropped, 0);
}

#[test]
fn event_bus_reports_queue_age_when_a_paused_consumer_resumes() {
    let bus = EventBus::new(EventBusDiagnosticsMode::Enabled);
    let events = bus
        .subscribe(
            "runtime.age",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    bus.try_publish(EngineEvent {
        topic: "runtime.age".to_string(),
        payload: Value::Null,
    })
    .expect("event admission");

    std::thread::sleep(Duration::from_millis(5));
    events.recv().unwrap();

    let report = bus.diagnostic_report();
    assert_eq!(report.queue_age_samples, 1);
    assert!(report.total_queue_age_ms >= 1.0);
    assert!(report.max_queue_age_ms >= 1.0);
    assert_eq!(report.queued, 0);
}

#[test]
fn event_bus_default_samples_routine_timings_but_keeps_exact_counters() {
    const EVENT_COUNT: u64 = 129;

    let bus = EventBus::default();
    let events = bus
        .subscribe(
            "runtime.sampled",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    for sequence in 0..EVENT_COUNT {
        bus.try_publish(EngineEvent {
            topic: "runtime.sampled".to_string(),
            payload: serde_json::json!({ "sequence": sequence }),
        })
        .expect("event admission");
        events.recv().unwrap();
    }

    let report = bus.diagnostic_report();
    assert!(report.enabled);
    assert_eq!(
        report.routine_timing_sample_interval,
        DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL.get()
    );
    assert_eq!(report.published, EVENT_COUNT);
    assert_eq!(report.delivered, EVENT_COUNT);
    assert_eq!(report.queued, 0);
    assert_eq!(report.publish_samples, 3);
    assert_eq!(report.queue_age_samples, 3);
}

#[test]
fn event_bus_explicit_sampling_uses_independent_publish_and_queue_sequences() {
    let bus = EventBus::new(EventBusDiagnosticsMode::Sampled {
        every: NonZeroU64::new(2).unwrap(),
    });
    let events = bus
        .subscribe(
            "runtime.sampled",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    for _ in 0..5 {
        bus.try_publish(EngineEvent {
            topic: "runtime.sampled".to_string(),
            payload: Value::Null,
        })
        .expect("event admission");
        events.recv().unwrap();
    }

    let report = bus.diagnostic_report();
    assert_eq!(report.routine_timing_sample_interval, 2);
    assert_eq!(report.publish_samples, 3);
    assert_eq!(report.queue_age_samples, 3);
}

// 订阅者不拥有总线；最后一个总线所有者释放后，阻塞和轮询接收都必须观察到断开。
#[test]
fn event_subscription_disconnects_when_the_last_event_bus_owner_drops() {
    let bus = EventBus::default();
    let polling = bus
        .subscribe(
            "runtime.shutdown",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let blocking = bus
        .subscribe(
            "runtime.shutdown",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
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
    let events = bus
        .subscribe(
            "runtime.long_wait",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let waiter = std::thread::spawn(move || events.recv_timeout(Duration::MAX));
    let deadline = Instant::now() + Duration::from_secs(1);
    while bus.diagnostic_report().waiting_receivers != 1 {
        assert!(
            Instant::now() < deadline,
            "overflowing timeout receiver returned instead of waiting"
        );
        std::thread::yield_now();
    }

    bus.try_publish(EngineEvent {
        topic: "runtime.long_wait".to_string(),
        payload: serde_json::json!({ "arrived": true }),
    })
    .expect("event admission");
    assert_eq!(
        waiter
            .join()
            .expect("long-timeout waiter should exit")
            .expect("long-timeout waiter should receive the event")
            .decode_payload()
            .unwrap()["arrived"],
        true
    );
}

#[test]
fn core_runtime_exposes_its_live_event_bus_diagnostics() {
    let runtime = CoreRuntime::new();
    let events = runtime
        .subscribe_events(
            "runtime.metrics",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    runtime
        .try_publish_event("runtime.metrics", serde_json::json!({ "frame": 1 }))
        .expect("event admission");

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
    let events = bus
        .subscribe(
            "runtime.silent",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    bus.try_publish(EngineEvent {
        topic: "runtime.silent".to_string(),
        payload: Value::Null,
    })
    .expect("event admission");
    events.recv().unwrap();

    let report = bus.diagnostic_report();
    assert!(!report.enabled);
    assert_eq!(report.routine_timing_sample_interval, 0);
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
    let events = bus
        .subscribe(
            "runtime.uncontended",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");

    bus.try_publish(EngineEvent {
        topic: "runtime.uncontended".to_string(),
        payload: Value::Null,
    })
    .expect("event admission");
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
    let events = bus
        .subscribe(
            "runtime.contended",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
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
        publisher_bus
            .try_publish(EngineEvent {
                topic: "runtime.contended".to_string(),
                payload: Value::Null,
            })
            .expect("event admission");
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

// 同一主题的多个订阅者必须看见相同全序，各发布者自身顺序也不能被扇出交错打乱。
#[test]
fn event_bus_concurrent_same_topic_publishers_share_one_exact_fanout_interleaving() {
    const PUBLISHER_COUNT: usize = 4;
    const EVENTS_PER_PUBLISHER: usize = 128;

    let bus = Arc::new(EventBus::default());
    let first_events = bus
        .subscribe(
            "runtime.concurrent",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let second_events = bus
        .subscribe(
            "runtime.concurrent",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let start = Arc::new(Barrier::new(PUBLISHER_COUNT + 1));
    let publishers = (0..PUBLISHER_COUNT)
        .map(|producer| {
            let bus = Arc::clone(&bus);
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                start.wait();
                for sequence in 0..EVENTS_PER_PUBLISHER {
                    bus.try_publish(EngineEvent {
                        topic: "runtime.concurrent".to_string(),
                        payload: serde_json::json!({
                            "producer": producer,
                            "sequence": sequence,
                        }),
                    })
                    .expect("event admission");
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
                    event.decode_payload().unwrap()["producer"]
                        .as_u64()
                        .unwrap(),
                    event.decode_payload().unwrap()["sequence"]
                        .as_u64()
                        .unwrap(),
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

// 订阅等待只应阻塞所属主题，不能将总线的全局主题表锁扩展到另一主题的发布。
#[test]
fn event_bus_allows_another_topic_to_progress_while_subscribe_waits_on_delivery() {
    let bus = Arc::new(EventBus::default());
    let blocked_events = bus
        .subscribe(
            "runtime.blocked",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let free_events = bus
        .subscribe(
            "runtime.free",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
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
        subscribe_bus
            .subscribe_after_reservation_for_test(
                "runtime.blocked",
                EngineEventDeliveryPolicy::Reliable {
                    limits: crate::core::framework::events::EventRetentionLimits::default(),
                },
                || {
                    subscribe_reserved.wait();
                },
            )
            .expect("bounded subscription")
    });
    reservation_reached.wait();

    let (progress_sender, progress_receiver) = mpsc::channel();
    let free_bus = Arc::clone(&bus);
    let free_publisher = std::thread::spawn(move || {
        free_bus
            .try_publish(EngineEvent {
                topic: "runtime.free".to_string(),
                payload: serde_json::json!({ "progress": true }),
            })
            .expect("event admission");
        progress_sender.send(()).unwrap();
    });
    let progress = progress_receiver.recv_timeout(Duration::from_secs(1));
    release_lock.wait();
    holder.join().unwrap();
    let added_events = subscriber.join().unwrap();
    free_publisher.join().unwrap();
    progress.expect("unrelated topic must progress while subscribe waits on another topic");
    assert_eq!(
        free_events.recv().unwrap().decode_payload().unwrap()["progress"],
        true
    );

    bus.try_publish(EngineEvent {
        topic: "runtime.blocked".to_string(),
        payload: serde_json::json!({ "released": true }),
    })
    .expect("event admission");
    assert_eq!(
        blocked_events.recv().unwrap().decode_payload().unwrap()["released"],
        true
    );
    assert_eq!(
        added_events.recv().unwrap().decode_payload().unwrap()["released"],
        true
    );
}

// 新订阅的预留必须阻止最后一个旧订阅删除主题，否则新句柄会留在已脱离主题表的对象上。
#[test]
fn event_bus_reservation_prevents_last_drop_from_orphaning_a_new_subscription() {
    let bus = Arc::new(EventBus::default());
    let anchor = bus
        .subscribe(
            "runtime.race",
            EngineEventDeliveryPolicy::Reliable {
                limits: crate::core::framework::events::EventRetentionLimits::default(),
            },
        )
        .expect("bounded subscription");
    let reserved = Arc::new(Barrier::new(2));
    let continue_subscription = Arc::new(Barrier::new(2));
    let subscribe_bus = Arc::clone(&bus);
    let subscribe_reserved = Arc::clone(&reserved);
    let subscribe_continue = Arc::clone(&continue_subscription);
    let subscriber = std::thread::spawn(move || {
        subscribe_bus
            .subscribe_after_reservation_for_test(
                "runtime.race",
                EngineEventDeliveryPolicy::Reliable {
                    limits: crate::core::framework::events::EventRetentionLimits::default(),
                },
                || {
                    subscribe_reserved.wait();
                    subscribe_continue.wait();
                },
            )
            .expect("bounded subscription")
    });

    reserved.wait();
    drop(anchor);
    assert_eq!(bus.diagnostic_report().topics, 1);
    assert_eq!(bus.diagnostic_report().subscribers, 0);
    continue_subscription.wait();

    let events = subscriber.join().unwrap();
    bus.try_publish(EngineEvent {
        topic: "runtime.race".to_string(),
        payload: serde_json::json!({ "iteration": 1 }),
    })
    .expect("event admission");
    assert_eq!(
        events
            .recv_timeout(Duration::from_secs(1))
            .expect("reserved subscription must remain registered")
            .decode_payload()
            .unwrap()["iteration"],
        1
    );
}
