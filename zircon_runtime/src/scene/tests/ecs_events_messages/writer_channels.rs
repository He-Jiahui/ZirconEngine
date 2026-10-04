//! The public Param tuple owns disjoint typed channels throughout one callback.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::*;
use crate::scene::ecs::SystemParamError;

#[derive(Debug, PartialEq, Eq)]
struct OtherEvent(u32);

#[test]
fn distinct_event_writers_reuse_both_channels_and_preserve_observer_order() {
    let mut world = World::empty();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let first = observed.clone();
    let second = observed.clone();
    world
        .observe_event_delivery::<FrameEvent, _>(move |event| {
            first.lock().unwrap().push((1, event.0));
            false
        })
        .unwrap();
    world
        .observe_event_delivery::<FrameEvent, _>(move |event| {
            second.lock().unwrap().push((2, event.0));
            true
        })
        .unwrap();
    type Writers = (EventWriterParam<FrameEvent>, EventWriterParam<OtherEvent>);
    let mut writers = SystemState::<Writers>::new(&mut world).unwrap();

    writers.run(&mut world, |(mut a, mut b)| {
        assert!(!a.send(FrameEvent(10)));
        assert!(b.send(OtherEvent(20)));
        assert_eq!(a.send_batch([FrameEvent(11), FrameEvent(12)]), 2);
        assert_eq!(b.send_batch([OtherEvent(21), OtherEvent(22)]), 2);
        assert!(!a.send(FrameEvent(13)));
    });
    assert_eq!(
        *observed.lock().unwrap(),
        vec![
            (1, 10),
            (2, 10),
            (1, 11),
            (2, 11),
            (1, 12),
            (2, 12),
            (1, 13),
            (2, 13)
        ]
    );
    assert!(world.events::<FrameEvent>().unwrap().is_empty());
    assert!(world.events::<OtherEvent>().unwrap().is_empty());
    world.update_all_events();
    assert_eq!(world.event_store().last_update_channel_visits(), 2);
    assert_eq!(
        world
            .events::<FrameEvent>()
            .unwrap()
            .iter()
            .map(|event| event.0)
            .collect::<Vec<_>>(),
        vec![10, 11, 12, 13]
    );
    assert_eq!(
        world
            .events::<OtherEvent>()
            .unwrap()
            .iter()
            .map(|event| event.0)
            .collect::<Vec<_>>(),
        vec![20, 21, 22]
    );
    assert_eq!(
        world.event_reader_count(world.event_type_id::<FrameEvent>().unwrap()),
        Some(2)
    );
}

#[test]
fn unused_event_writers_and_empty_batches_do_not_activate_maintenance() {
    let mut world = World::empty();
    type Writers = (EventWriterParam<FrameEvent>, EventWriterParam<OtherEvent>);
    let mut writers = SystemState::<Writers>::new(&mut world).unwrap();
    writers.run(&mut world, |(mut a, mut b)| {
        assert_eq!(a.send_batch(std::iter::empty()), 0);
        assert_eq!(b.send_batch(std::iter::empty()), 0);
    });
    assert_eq!(world.event_store().active_channel_count(), 0);
    world.update_all_events();
    assert_eq!(world.event_store().last_update_channel_visits(), 0);
    assert!(world.events::<FrameEvent>().unwrap().is_empty());
    assert!(world.events::<OtherEvent>().unwrap().is_empty());
}

#[test]
fn typed_writers_allow_other_channel_readers_and_reject_same_channel_aliases() {
    let mut world = World::empty();
    world.send_event(OtherEvent(7));
    world.update_all_events();
    world.send_message(WeightedMessage {
        value: 8,
        retention_bytes: 4,
    });
    type Params = (
        EventReaderParam<OtherEvent>,
        EventWriterParam<FrameEvent>,
        MessageReaderParam<WeightedMessage>,
        MessageWriterParam<RetainedMessage>,
    );
    let mut state = SystemState::<Params>::new(&mut world).unwrap();
    state.run(
        &mut world,
        |(mut events, mut event_writer, mut messages, mut message_writer)| {
            let retained_events = events.iter().collect::<Vec<_>>();
            let retained_messages = messages
                .read()
                .map(|(_, message)| message)
                .collect::<Vec<_>>();
            assert!(event_writer.send(FrameEvent(9)));
            assert_eq!(message_writer.write(RetainedMessage(10)).id(), 0);
            assert_eq!(retained_events[0].0, 7);
            assert_eq!(retained_messages[0].value, 8);
            assert!(event_writer.send(FrameEvent(11)));
            assert_eq!(message_writer.write(RetainedMessage(12)).id(), 1);
            assert_eq!(retained_events[0].0, 7);
            assert_eq!(retained_messages[0].value, 8);
        },
    );
    let event_conflict = SystemParamError::TupleElement {
        index: 1,
        parameter_type: std::any::type_name::<EventWriterParam<FrameEvent>>(),
        source: Box::new(SystemParamError::ConflictingEventAccess {
            type_name: std::any::type_name::<FrameEvent>(),
        }),
    };
    assert_eq!(
        SystemState::<(EventWriterParam<FrameEvent>, EventWriterParam<FrameEvent>)>::new(
            &mut world
        )
        .err(),
        Some(event_conflict.clone())
    );
    assert_eq!(
        SystemState::<(EventReaderParam<FrameEvent>, EventWriterParam<FrameEvent>)>::new(
            &mut world
        )
        .err(),
        Some(event_conflict)
    );
    let message_conflict = SystemParamError::TupleElement {
        index: 1,
        parameter_type: std::any::type_name::<MessageWriterParam<RetainedMessage>>(),
        source: Box::new(SystemParamError::ConflictingMessageAccess {
            type_name: std::any::type_name::<RetainedMessage>(),
        }),
    };
    assert_eq!(
        SystemState::<(
            MessageWriterParam<RetainedMessage>,
            MessageWriterParam<RetainedMessage>
        )>::new(&mut world)
        .err(),
        Some(message_conflict.clone())
    );
    assert_eq!(
        SystemState::<(
            MessageReaderParam<RetainedMessage>,
            MessageWriterParam<RetainedMessage>
        )>::new(&mut world)
        .err(),
        Some(message_conflict)
    );
}

#[test]
fn distinct_message_writers_keep_ids_byte_budgets_and_current_frame_age() {
    let mut world = World::empty();
    world.configure_message_retention::<RetainedMessage>(MessageRetention::new(8, usize::MAX, 1));
    world.configure_message_retention::<WeightedMessage>(MessageRetention::new(8, 6, 1));
    world.advance_messages();
    world.advance_messages();
    type Writers = (
        MessageWriterParam<RetainedMessage>,
        MessageWriterParam<WeightedMessage>,
    );
    let mut writers = SystemState::<Writers>::new(&mut world).unwrap();
    writers.run(&mut world, |(mut a, mut b)| {
        assert_eq!(a.write(RetainedMessage(1)).id(), 0);
        assert_eq!(
            b.write(WeightedMessage {
                value: 10,
                retention_bytes: 6
            })
            .id(),
            0
        );
        assert_eq!(
            a.write_batch([RetainedMessage(2), RetainedMessage(3)])
                .iter()
                .map(|id| id.id())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            b.write(WeightedMessage {
                value: 11,
                retention_bytes: 6
            })
            .id(),
            1
        );
    });
    assert_eq!(
        world
            .message_retention_metrics::<WeightedMessage>()
            .unwrap()
            .retained_bytes,
        6
    );
    assert_eq!(
        world
            .message_retention_metrics::<WeightedMessage>()
            .unwrap()
            .budget_dropped_entries,
        1
    );
    let mut reader = SystemState::<(
        MessageReaderParam<RetainedMessage>,
        MessageReaderParam<WeightedMessage>,
    )>::new(&mut world)
    .unwrap();
    reader.run(&mut world, |(mut a, mut b)| {
        assert_eq!(
            a.read()
                .map(|(id, message)| (id.id(), message.0))
                .collect::<Vec<_>>(),
            vec![(0, 1), (1, 2), (2, 3)]
        );
        assert_eq!(
            b.read()
                .map(|(id, message)| (id.id(), message.value))
                .collect::<Vec<_>>(),
            vec![(1, 11)]
        );
        assert_eq!(b.dropped_count(), 1);
    });
    world.advance_messages();
    assert_eq!(world.last_message_advance_channel_visits(), 2);
    assert_eq!(world.messages::<RetainedMessage>().unwrap().len(), 3);
    assert_eq!(world.messages::<WeightedMessage>().unwrap().len(), 1);
    world.advance_messages();
    assert_eq!(world.last_message_advance_channel_visits(), 2);
    assert_eq!(
        world
            .message_retention_metrics::<RetainedMessage>()
            .unwrap()
            .age_dropped_entries,
        3
    );
    assert_eq!(
        world
            .message_retention_metrics::<WeightedMessage>()
            .unwrap()
            .age_dropped_entries,
        1
    );
    world.advance_messages();
    assert_eq!(world.last_message_advance_channel_visits(), 0);
    writers.run(&mut world, |(mut a, mut b)| {
        assert_eq!(a.write(RetainedMessage(4)).id(), 3);
        assert_eq!(
            b.write(WeightedMessage {
                value: 12,
                retention_bytes: 6
            })
            .id(),
            2
        );
    });
}

#[test]
fn message_writer_preparation_stays_private_until_the_first_even_empty_batch() {
    let mut world = World::empty();
    type Writers = (
        MessageWriterParam<RetainedMessage>,
        MessageWriterParam<WeightedMessage>,
    );
    let mut writers = SystemState::<Writers>::new(&mut world).unwrap();
    assert!(world.messages::<RetainedMessage>().is_none());
    assert!(world.messages::<WeightedMessage>().is_none());
    assert!(world.message_store_mut().registered_type_names().is_empty());
    writers.run(&mut world, |(_a, _b)| {});
    world.advance_messages();
    assert_eq!(world.last_message_advance_channel_visits(), 0);
    assert!(world.messages::<RetainedMessage>().is_none());
    assert!(world.message_store_mut().registered_type_names().is_empty());

    writers.run(&mut world, |(mut a, _b)| {
        assert!(a.write_batch(std::iter::empty()).is_empty());
    });
    assert!(world.messages::<RetainedMessage>().unwrap().is_empty());
    assert!(world.messages::<WeightedMessage>().is_none());
    assert_eq!(
        world.message_store_mut().registered_type_names(),
        vec![std::any::type_name::<RetainedMessage>()]
    );
    world.advance_messages();
    assert_eq!(world.last_message_advance_channel_visits(), 1);
    world.advance_messages();
    assert_eq!(world.last_message_advance_channel_visits(), 0);
    writers.run(&mut world, |(mut a, mut b)| {
        assert_eq!(a.write(RetainedMessage(1)).id(), 0);
        assert_eq!(
            b.write(WeightedMessage {
                value: 2,
                retention_bytes: 3
            })
            .id(),
            0
        );
    });
}

#[test]
fn disjoint_typed_writer_grants_keep_their_existing_scoped_thread_capability() {
    let mut world = World::empty();
    type Params = (
        EventWriterParam<FrameEvent>,
        EventWriterParam<OtherEvent>,
        MessageWriterParam<RetainedMessage>,
        MessageWriterParam<WeightedMessage>,
    );
    let mut state = SystemState::<Params>::new(&mut world).unwrap();
    state.run(
        &mut world,
        |(mut event_a, mut event_b, mut message_a, mut message_b)| {
            std::thread::scope(|scope| {
                let a = scope.spawn(move || {
                    assert!(event_a.send(FrameEvent(1)));
                    assert_eq!(message_a.write(RetainedMessage(2)).id(), 0);
                });
                let b = scope.spawn(move || {
                    assert!(event_b.send(OtherEvent(3)));
                    assert_eq!(
                        message_b
                            .write(WeightedMessage {
                                value: 4,
                                retention_bytes: 3
                            })
                            .id(),
                        0
                    );
                });
                a.join().unwrap();
                b.join().unwrap();
            });
        },
    );
    world.update_all_events();
    assert_eq!(world.event_store().last_update_channel_visits(), 2);
    assert_eq!(
        world
            .events::<FrameEvent>()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .0,
        1
    );
    assert_eq!(
        world
            .events::<OtherEvent>()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .0,
        3
    );
    assert_eq!(
        world
            .messages::<RetainedMessage>()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .1
             .0,
        2
    );
    assert_eq!(
        world
            .messages::<WeightedMessage>()
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .1
            .value,
        4
    );
}

struct DroppedEvent(Arc<AtomicUsize>);

impl Drop for DroppedEvent {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

struct DroppedMessage(Arc<AtomicUsize>);

impl Message for DroppedMessage {}

impl Drop for DroppedMessage {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn typed_channel_clear_and_world_drop_release_each_payload_once() {
    let event_drops = Arc::new(AtomicUsize::new(0));
    let message_drops = Arc::new(AtomicUsize::new(0));
    let mut world = World::empty();
    type Params = (
        EventWriterParam<DroppedEvent>,
        MessageWriterParam<DroppedMessage>,
    );
    let mut state = SystemState::<Params>::new(&mut world).unwrap();
    state.run(&mut world, |(mut events, mut messages)| {
        assert!(events.send(DroppedEvent(event_drops.clone())));
        messages.write(DroppedMessage(message_drops.clone()));
    });
    assert_eq!(event_drops.load(Ordering::Relaxed), 0);
    assert_eq!(message_drops.load(Ordering::Relaxed), 0);
    world.event_store_mut().events_mut::<DroppedEvent>().clear();
    world.message_store_mut().clear::<DroppedMessage>();
    assert_eq!(event_drops.load(Ordering::Relaxed), 1);
    assert_eq!(message_drops.load(Ordering::Relaxed), 1);

    state.run(&mut world, |(mut events, mut messages)| {
        assert!(events.send(DroppedEvent(event_drops.clone())));
        messages.write(DroppedMessage(message_drops.clone()));
    });
    drop(state);
    drop(world);
    assert_eq!(event_drops.load(Ordering::Relaxed), 2);
    assert_eq!(message_drops.load(Ordering::Relaxed), 2);
}
