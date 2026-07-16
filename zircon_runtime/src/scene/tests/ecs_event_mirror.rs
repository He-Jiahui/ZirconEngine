use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use serde::Serialize;

use crate::scene::{RuntimeEventMirrorError, RuntimeEventMirrorRegistration, SceneError, World};

const EVENT_ID: &str = "tests.events.mirrored";
const PAYLOAD_SCHEMA: &str = "zircon.tests.mirrored.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MirroredEvent {
    value: u32,
}

#[test]
fn runtime_event_mirror_is_schema_bound_current_only_and_reference_counted() {
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
