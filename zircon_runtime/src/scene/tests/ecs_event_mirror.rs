use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use serde::Serialize;

use crate::scene::{
    RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS, RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
    RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS, RuntimeEventMirrorError, RuntimeEventMirrorRegistration,
    SceneError, World,
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
    assert!(
        world
            .drain_runtime_event_mirror(&mut first)
            .unwrap()
            .is_empty()
    );

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
    let source = include_str!("../event_mirror/subscription.rs");
    let raw_page_drain_source = source
        .split("pub(crate) fn drain_payloads")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn drain(").next())
        .expect("runtime event mirror raw page drain source");
    let connected_guard = raw_page_drain_source
        .find("if !self.connected")
        .expect("runtime event mirror connected guard");

    assert!(raw_page_drain_source.contains("match self.erased.drain_payloads()"));
    assert!(!raw_page_drain_source[..connected_guard].contains("event_id.clone()"));
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
    assert!(
        world
            .unsubscribe_runtime_event_mirror(&mut subscription)
            .unwrap()
    );
    assert_eq!(readers.load(Ordering::SeqCst), 0);
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
    assert!(
        world
            .unsubscribe_runtime_event_mirror(&mut subscription)
            .unwrap()
    );
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
    assert!(
        world
            .drain_runtime_event_mirror(&mut subscription)
            .unwrap()
            .is_empty()
    );
}
