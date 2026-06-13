use crate::scene::ecs::{
    EventReaderParam, EventWriterParam, Message, MessageReaderParam, MessageWriterParam,
    SystemStage, SystemState,
};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct FrameEvent(u32);

#[derive(Debug, PartialEq, Eq)]
struct RetainedMessage(u32);

impl Message for RetainedMessage {}

#[test]
fn events_require_explicit_update_and_keep_next_queue_hidden() {
    let mut world = World::empty();
    let mut writer = SystemState::<EventWriterParam<FrameEvent>>::new(&mut world).unwrap();
    let mut reader = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();

    writer.run(&mut world, |mut events| events.send(FrameEvent(1)));
    let before_update = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert!(before_update.is_empty());

    world.update_events::<FrameEvent>();
    let current = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(current, vec![1]);

    writer.run(&mut world, |mut events| events.send(FrameEvent(2)));
    let same_generation = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert!(same_generation.is_empty());

    world.update_events::<FrameEvent>();
    let next_generation = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(next_generation, vec![2]);
}

#[test]
fn first_stage_updates_all_registered_event_channels() {
    let mut world = World::empty();
    let mut reader = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();

    assert!(world.send_event(FrameEvent(11)));
    let before_first = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert!(before_first.is_empty());

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    let after_first = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(after_first, vec![11]);
}

#[test]
fn clear_events_prunes_current_and_next_event_queues() {
    let mut world = World::empty();
    let mut writer = SystemState::<EventWriterParam<FrameEvent>>::new(&mut world).unwrap();
    let mut reader = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();

    writer.run(&mut world, |mut events| events.send(FrameEvent(1)));
    world.update_events::<FrameEvent>();
    writer.run(&mut world, |mut events| events.send(FrameEvent(2)));
    world.clear_events::<FrameEvent>();
    world.update_events::<FrameEvent>();

    let after_clear = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert!(after_clear.is_empty());

    writer.run(&mut world, |mut events| events.send(FrameEvent(3)));
    world.update_events::<FrameEvent>();
    let after_reset = reader.run(&mut world, |mut events| {
        events.iter().map(|event| event.0).collect::<Vec<_>>()
    });
    assert_eq!(after_reset, vec![3]);
}

#[test]
fn messages_are_retained_until_explicit_clear_independent_of_event_updates() {
    let mut world = World::empty();
    let mut writer = SystemState::<MessageWriterParam<RetainedMessage>>::new(&mut world).unwrap();
    let mut first_reader =
        SystemState::<MessageReaderParam<RetainedMessage>>::new(&mut world).unwrap();

    let ids = writer.run(&mut world, |mut messages| {
        messages.write_batch([RetainedMessage(1), RetainedMessage(2)])
    });
    assert_eq!(ids[0].id(), 0);
    assert_eq!(ids[1].id(), 1);

    world.update_events::<FrameEvent>();
    let first_read = first_reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(first_read, vec![(0, 1), (1, 2)]);

    let reread = first_reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>()
    });
    assert!(reread.is_empty());

    let mut second_reader =
        SystemState::<MessageReaderParam<RetainedMessage>>::new(&mut world).unwrap();
    let retained_for_new_reader = second_reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(retained_for_new_reader, vec![(0, 1), (1, 2)]);

    world.clear_messages::<RetainedMessage>();
    let next_id = world.send_message(RetainedMessage(3));
    assert_eq!(next_id.id(), 2);
    let after_clear = first_reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(after_clear, vec![(2, 3)]);
}

#[test]
fn event_and_message_clear_boundaries_do_not_cross_channels() {
    let mut world = World::empty();
    let _event_reader = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();

    world.send_event(FrameEvent(1));
    world.send_message(RetainedMessage(9));
    world.update_events::<FrameEvent>();
    world.clear_events::<FrameEvent>();

    assert_eq!(
        world
            .messages::<RetainedMessage>()
            .map(|messages| messages.len()),
        Some(1)
    );

    world.send_event(FrameEvent(2));
    world.update_events::<FrameEvent>();
    world.clear_messages::<RetainedMessage>();

    assert_eq!(
        world.events::<FrameEvent>().map(|events| events.len()),
        Some(1)
    );
    assert_eq!(
        world
            .messages::<RetainedMessage>()
            .map(|messages| messages.len()),
        Some(0)
    );
}
