use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

use serde::Serialize;

use crate::scene::{
    RuntimeEventMirrorError, RuntimeEventMirrorRegistration, SceneError, World,
    RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS, RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
    RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS,
};

const EVENT_ID: &str = "tests.events.mirrored";
const PAYLOAD_SCHEMA: &str = "zircon.tests.mirrored.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MirroredEvent {
    value: u32,
}

#[derive(Clone, Debug, Serialize)]
struct OversizedMirroredEvent {
    payload: String,
}

#[test]
fn runtime_event_mirror_is_schema_bound_send_boundary_and_reference_counted() {
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_callback = readers.clone();
    let registration =
        RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA)
            .with_reader_count_callback(move |_world, count| {
                readers_for_callback.store(count, Ordering::SeqCst);
                Ok(())
            });
    let mut world = World::empty();
    world.register_runtime_event_mirror(registration).unwrap();

    let event_type_id = world.event_store_mut().register_reader::<MirroredEvent>();
    assert_eq!(world.event_reader_count(event_type_id), Some(1));

    world.send_event(MirroredEvent { value: 1 });
    world.update_events::<MirroredEvent>();
    let mut first = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    assert_eq!(world.event_reader_count(event_type_id), Some(2));
    assert_eq!(readers.load(Ordering::SeqCst), 1);
    assert!(world
        .drain_runtime_event_mirror(&mut first)
        .unwrap()
        .is_empty());

    world.send_event(MirroredEvent { value: 2 });
    world.update_events::<MirroredEvent>();
    assert_eq!(
        world.drain_runtime_event_mirror(&mut first).unwrap(),
        [serde_json::json!({"value": 2})]
    );

    let mut second = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    assert_eq!(readers.load(Ordering::SeqCst), 2);
    assert!(world.unsubscribe_runtime_event_mirror(&mut first).unwrap());
    assert_eq!(readers.load(Ordering::SeqCst), 1);
    assert!(world.unsubscribe_runtime_event_mirror(&mut second).unwrap());
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    assert_eq!(world.event_reader_count(event_type_id), Some(1));
}

#[test]
fn runtime_event_mirror_rejects_wrong_schema_and_duplicate_event_id() {
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<MirroredEvent>(
            EVENT_ID,
            PAYLOAD_SCHEMA,
        ))
        .unwrap();

    assert!(matches!(
        world.subscribe_runtime_event_mirror(EVENT_ID, "wrong.schema"),
        Err(RuntimeEventMirrorError::PayloadSchemaMismatch { .. })
    ));
    assert!(matches!(
        world.register_runtime_event_mirror(
            RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA,)
        ),
        Err(RuntimeEventMirrorError::DuplicateEventId { .. })
    ));
}

#[test]
fn successful_runtime_event_mirror_drain_does_not_clone_the_event_id() {
    let source = include_str!("../world/event_mirror.rs");
    let raw_page_drain_source = source
        .split("pub(crate) fn drain_runtime_event_mirror_payloads")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(crate) fn runtime_event_mirror_lifecycle_diagnostics")
                .next()
        })
        .expect("runtime event mirror raw page drain source");
    let disconnected_branch = raw_page_drain_source
        .find("unwrap_or_else")
        .expect("runtime event mirror disconnected branch");

    assert!(raw_page_drain_source.contains("drain_subscription_payloads"));
    assert!(!raw_page_drain_source[..disconnected_branch].contains("event_id.clone()"));
    assert!(!raw_page_drain_source.contains("serde_json::from_slice"));
}

#[test]
fn runtime_event_mirror_hardcuts_current_only_cursor_transport() {
    let source = include_str!("../event_mirror/subscription.rs");

    assert!(source.contains("world.observe_event_delivery::<E, _>"));
    assert!(source.contains("RuntimeEventMirrorQueue"));
    assert!(source.contains("RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS"));
    assert!(!source.contains("read_event_subscription"));
    assert!(!source.contains("collect::<Result<Vec"));
}

#[test]
fn runtime_event_mirror_unsubscribe_rolls_back_when_reader_callback_fails() {
    let fail_disconnect = Arc::new(AtomicBool::new(true));
    let fail_disconnect_for_callback = fail_disconnect.clone();
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_callback = readers.clone();
    let registration =
        RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA)
            .with_reader_count_callback(move |_world, count| {
                if count == 0 && fail_disconnect_for_callback.load(Ordering::SeqCst) {
                    return Err(SceneError::EmptyNodeName);
                }
                readers_for_callback.store(count, Ordering::SeqCst);
                Ok(())
            });
    let mut world = World::empty();
    world.register_runtime_event_mirror(registration).unwrap();
    let mut subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    assert!(matches!(
        world.unsubscribe_runtime_event_mirror(&mut subscription),
        Err(RuntimeEventMirrorError::ReaderCountCallback { .. })
    ));
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    fail_disconnect.store(false, Ordering::SeqCst);
    assert!(world
        .unsubscribe_runtime_event_mirror(&mut subscription)
        .unwrap());
    assert_eq!(readers.load(Ordering::SeqCst), 0);
}

#[test]
fn dropped_runtime_event_mirrors_reclaim_with_a_live_record_hard_budget() {
    for subscription_count in [1_usize, 100, 10_000] {
        let callback_edges = Arc::new(AtomicUsize::new(0));
        let callback_edges_for_registration = Arc::clone(&callback_edges);
        let readers = Arc::new(AtomicU32::new(0));
        let readers_for_registration = Arc::clone(&readers);
        let mut world = World::empty();
        world
            .register_runtime_event_mirror(
                RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA)
                    .with_reader_count_callback(move |_world, count| {
                        callback_edges_for_registration.fetch_add(1, Ordering::SeqCst);
                        readers_for_registration.store(count, Ordering::SeqCst);
                        Ok(())
                    }),
            )
            .unwrap();
        let event_type_id = world.event_store_mut().register::<MirroredEvent>();
        let subscriptions = (0..subscription_count)
            .map(|_| {
                world
                    .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let connected = world
            .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
            .unwrap();
        assert_eq!(connected.live_subscriptions, subscription_count);
        assert_eq!(connected.pending_reclaims, 0);
        assert_eq!(connected.reclaim_budget, subscription_count);
        assert_eq!(connected.reader_count, subscription_count as u32);
        assert_eq!(
            world.event_reader_count(event_type_id),
            Some(subscription_count as u32)
        );

        drop(subscriptions);
        let pending = world
            .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
            .unwrap();
        assert_eq!(pending.live_subscriptions, subscription_count);
        assert_eq!(pending.pending_reclaims, subscription_count);
        assert_eq!(pending.reclaim_budget, subscription_count);

        let report = world.reclaim_dropped_runtime_event_mirrors();
        assert_eq!(report.attempted, subscription_count);
        assert_eq!(report.reclaimed, subscription_count);
        assert_eq!(report.retry_pending, 0);
        assert_eq!(report.callback_failures, 0);
        let reclaimed = world
            .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
            .unwrap();
        assert_eq!(reclaimed.live_subscriptions, 0);
        assert_eq!(reclaimed.pending_reclaims, 0);
        assert_eq!(reclaimed.reclaim_budget, 0);
        assert_eq!(reclaimed.reader_count, 0);
        assert_eq!(world.event_reader_count(event_type_id), Some(0));
        assert_eq!(readers.load(Ordering::SeqCst), 0);
        assert_eq!(
            callback_edges.load(Ordering::SeqCst),
            subscription_count + 1,
            "drop reclamation must publish one final reader-count edge per event"
        );

        let successor = world
            .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
            .unwrap();
        assert_eq!(world.event_reader_count(event_type_id), Some(1));
        drop(successor);
        assert_eq!(world.reclaim_dropped_runtime_event_mirrors().reclaimed, 1);
        assert_eq!(world.event_reader_count(event_type_id), Some(0));
    }
}

#[test]
fn dropped_runtime_event_mirror_callback_failure_retries_without_double_disconnect() {
    let fail_zero = Arc::new(AtomicBool::new(true));
    let fail_zero_for_registration = Arc::clone(&fail_zero);
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_registration = Arc::clone(&readers);
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(
            RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA)
                .with_reader_count_callback(move |_world, count| {
                    if count == 0 && fail_zero_for_registration.load(Ordering::SeqCst) {
                        return Err(SceneError::EmptyNodeName);
                    }
                    readers_for_registration.store(count, Ordering::SeqCst);
                    Ok(())
                }),
        )
        .unwrap();
    let event_type_id = world.event_store_mut().register::<MirroredEvent>();
    let subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    drop(subscription);

    let failed = world.reclaim_dropped_runtime_event_mirrors();
    assert_eq!(failed.attempted, 1);
    assert_eq!(failed.reclaimed, 0);
    assert_eq!(failed.retry_pending, 1);
    assert_eq!(failed.callback_failures, 1);
    assert_eq!(readers.load(Ordering::SeqCst), 1);
    assert_eq!(world.event_reader_count(event_type_id), Some(1));
    let retained = world
        .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
        .unwrap();
    assert_eq!(retained.live_subscriptions, 1);
    assert_eq!(retained.pending_reclaims, 1);

    fail_zero.store(false, Ordering::SeqCst);
    let recovered = world.reclaim_dropped_runtime_event_mirrors();
    assert_eq!(recovered.attempted, 1);
    assert_eq!(recovered.reclaimed, 1);
    assert_eq!(recovered.retry_pending, 0);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    assert_eq!(world.event_reader_count(event_type_id), Some(0));
}

#[test]
fn runtime_event_mirror_rejects_foreign_world_ownership_without_disconnect() {
    let registration =
        RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA);
    let mut owner = World::empty();
    owner
        .register_runtime_event_mirror(registration.clone())
        .unwrap();
    let mut foreign = World::empty();
    foreign.register_runtime_event_mirror(registration).unwrap();
    let mut subscription = owner
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();

    assert!(!foreign
        .unsubscribe_runtime_event_mirror(&mut subscription)
        .unwrap());
    assert_eq!(
        owner
            .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
            .unwrap()
            .live_subscriptions,
        1
    );
    assert!(owner
        .unsubscribe_runtime_event_mirror(&mut subscription)
        .unwrap());
}

#[test]
fn runtime_event_mirror_mixes_explicit_and_drop_reclaim_without_double_retirement() {
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<MirroredEvent>(
            EVENT_ID,
            PAYLOAD_SCHEMA,
        ))
        .unwrap();
    let mut explicit = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    let dropped = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();

    assert!(world
        .unsubscribe_runtime_event_mirror(&mut explicit)
        .unwrap());
    drop(explicit);
    drop(dropped);

    let report = world.reclaim_dropped_runtime_event_mirrors();
    assert_eq!(report.attempted, 1);
    assert_eq!(report.reclaimed, 1);
    let diagnostics = world
        .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
        .unwrap();
    assert_eq!(diagnostics.live_subscriptions, 0);
    assert_eq!(diagnostics.pending_reclaims, 0);
}

#[test]
fn runtime_event_mirror_ignores_stale_generation_after_shutdown_slot_reuse() {
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<MirroredEvent>(
            EVENT_ID,
            PAYLOAD_SCHEMA,
        ))
        .unwrap();
    let stale = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();

    assert_eq!(world.shutdown_runtime_event_mirrors().reclaimed, 1);
    let successor = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    drop(stale);
    assert_eq!(
        world
            .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
            .unwrap()
            .pending_reclaims,
        0
    );

    drop(successor);
    assert_eq!(
        world
            .runtime_event_mirror_lifecycle_diagnostics(EVENT_ID)
            .unwrap()
            .pending_reclaims,
        1
    );
    assert_eq!(world.reclaim_dropped_runtime_event_mirrors().reclaimed, 1);
}

#[test]
fn world_drop_quiesces_live_runtime_event_mirror_tokens() {
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_callback = Arc::clone(&readers);
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(
            RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA)
                .with_reader_count_callback(move |_world, count| {
                    readers_for_callback.store(count, Ordering::SeqCst);
                    Ok(())
                }),
        )
        .unwrap();
    let subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    drop(world);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    drop(subscription);
}

#[test]
fn runtime_event_mirror_shutdown_reports_callback_failure_until_retry_succeeds() {
    let fail_zero = Arc::new(AtomicBool::new(true));
    let fail_zero_for_callback = Arc::clone(&fail_zero);
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_callback = Arc::clone(&readers);
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(
            RuntimeEventMirrorRegistration::typed::<MirroredEvent>(EVENT_ID, PAYLOAD_SCHEMA)
                .with_reader_count_callback(move |_world, count| {
                    if count == 0 && fail_zero_for_callback.load(Ordering::SeqCst) {
                        return Err(SceneError::EmptyNodeName);
                    }
                    readers_for_callback.store(count, Ordering::SeqCst);
                    Ok(())
                }),
        )
        .unwrap();
    let subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();

    let failed = world.shutdown_runtime_event_mirrors();
    assert_eq!(failed.callback_failures, 1);
    assert_eq!(failed.retry_pending, 1);
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    fail_zero.store(false, Ordering::SeqCst);
    let recovered = world.shutdown_runtime_event_mirrors();
    assert_eq!(recovered.reclaimed, 1);
    assert_eq!(recovered.retry_pending, 0);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    drop(subscription);
}

#[test]
fn world_driver_reclaims_dropped_event_mirrors_before_the_first_schedule_stage() {
    let source = include_str!("../module/world_driver.rs");
    let reclaim = source
        .find("reclaim_dropped_runtime_event_mirrors")
        .expect("WorldDriver must process runtime mirror reclaim intents");
    let schedule = source
        .find("for stage in schedule.stages()")
        .expect("WorldDriver schedule loop");

    assert!(reclaim < schedule);
}

#[test]
fn runtime_event_mirror_pages_persist_across_world_event_updates_without_loss() {
    const EVENT_COUNT: u32 = 10_000;

    let mut world = World::empty();
    world
        .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<MirroredEvent>(
            EVENT_ID,
            PAYLOAD_SCHEMA,
        ))
        .unwrap();
    let mut subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();

    for value in 0..EVENT_COUNT {
        assert!(world.send_event(MirroredEvent { value }));
    }
    world.update_events::<MirroredEvent>();

    let mut received = Vec::with_capacity(EVENT_COUNT as usize);
    loop {
        let page = world.drain_runtime_event_mirror(&mut subscription).unwrap();
        assert!(page.len() <= RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS);
        if page.is_empty() {
            break;
        }
        received.extend(
            page.into_iter()
                .map(|payload| payload["value"].as_u64().unwrap() as u32),
        );
        world.update_events::<MirroredEvent>();
    }

    assert_eq!(received, (0..EVENT_COUNT).collect::<Vec<_>>());
    assert!(world
        .unsubscribe_runtime_event_mirror(&mut subscription)
        .unwrap());
}

#[test]
fn runtime_event_mirror_queue_overflow_is_explicit_and_preserves_accepted_events() {
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<MirroredEvent>(
            EVENT_ID,
            PAYLOAD_SCHEMA,
        ))
        .unwrap();
    let mut subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();

    for value in 0..RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS as u32 {
        assert!(world.send_event(MirroredEvent { value }));
    }
    assert!(!world.send_event(MirroredEvent {
        value: RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS as u32,
    }));
    assert!(matches!(
        world.drain_runtime_event_mirror(&mut subscription),
        Err(RuntimeEventMirrorError::QueueOverflow {
            pending_events: RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS,
            max_events: RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS,
            ..
        })
    ));

    let first_page = world.drain_runtime_event_mirror(&mut subscription).unwrap();
    assert_eq!(first_page.len(), RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS);
    assert_eq!(first_page[0]["value"], 0);
}

#[test]
fn runtime_event_mirror_rejects_descriptors_that_cannot_fit_the_wire_budget() {
    let event_id = "e".repeat(129);
    let mut world = World::empty();

    assert!(matches!(
        world.register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<
            MirroredEvent,
        >(event_id.clone(), PAYLOAD_SCHEMA)),
        Err(RuntimeEventMirrorError::DescriptorTooLarge {
            event_id: actual,
            field: "event id",
            actual_bytes: 129,
            max_bytes: 128,
        }) if actual == event_id
    ));
}

#[test]
fn runtime_event_mirror_rejects_a_payload_larger_than_one_wire_page() {
    let mut world = World::empty();
    world
        .register_runtime_event_mirror(RuntimeEventMirrorRegistration::typed::<
            OversizedMirroredEvent,
        >(EVENT_ID, PAYLOAD_SCHEMA))
        .unwrap();
    let mut subscription = world
        .subscribe_runtime_event_mirror(EVENT_ID, PAYLOAD_SCHEMA)
        .unwrap();

    assert!(!world.send_event(OversizedMirroredEvent {
        payload: "x".repeat(RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES),
    }));
    assert!(matches!(
        world.drain_runtime_event_mirror(&mut subscription),
        Err(RuntimeEventMirrorError::PayloadTooLarge {
            max_payload_bytes: RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
            ..
        })
    ));
    assert!(world
        .drain_runtime_event_mirror(&mut subscription)
        .unwrap()
        .is_empty());
}
