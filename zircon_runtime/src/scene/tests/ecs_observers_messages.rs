use std::sync::{Arc, Mutex};

use crate::scene::components::Name;
use crate::scene::ecs::{
    Component, LifecycleEventKind, Message, MessageReaderParam, MessageWriterParam, SystemState,
};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct DamageEvent(u32);

#[derive(Debug, PartialEq, Eq)]
struct DamageMessage(u32);

impl Message for DamageMessage {}

#[test]
fn lifecycle_observers_report_insert_replace_remove_and_despawn_order() {
    let mut world = World::empty();
    let health_id = world.component_id::<Health>();
    let events = Arc::new(Mutex::new(Vec::new()));

    for kind in [
        LifecycleEventKind::Add,
        LifecycleEventKind::Insert,
        LifecycleEventKind::Replace,
        LifecycleEventKind::Remove,
        LifecycleEventKind::Despawn,
    ] {
        let events = events.clone();
        world.observe_component_lifecycle::<Health>(kind, move |_world, event| {
            events.lock().unwrap().push(format!(
                "{:?}:{}:{}",
                event.kind(),
                event.entity(),
                event.component_id().index()
            ));
        });
    }

    let entity = world
        .spawn((Name("Observed".to_string()), Health(1)))
        .unwrap();
    world.insert(entity, Health(2)).unwrap();
    world.remove::<Health>(entity).unwrap();
    world.insert(entity, Health(3)).unwrap();
    assert!(world.remove_entity(entity));

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            format!("Add:{entity}:{}", health_id.index()),
            format!("Insert:{entity}:{}", health_id.index()),
            format!("Replace:{entity}:{}", health_id.index()),
            format!("Insert:{entity}:{}", health_id.index()),
            format!("Remove:{entity}:{}", health_id.index()),
            format!("Add:{entity}:{}", health_id.index()),
            format!("Insert:{entity}:{}", health_id.index()),
            format!("Remove:{entity}:{}", health_id.index()),
            format!("Despawn:{entity}:{}", health_id.index()),
        ]
    );
}

#[test]
fn observer_handles_can_be_removed_before_later_triggers() {
    let mut world = World::empty();
    let events = Arc::new(Mutex::new(Vec::new()));
    let observer = {
        let events = events.clone();
        world.observe_component_lifecycle::<Health>(
            LifecycleEventKind::Add,
            move |_world, event| {
                events.lock().unwrap().push(event.entity());
            },
        )
    };

    let first = world.spawn((Name("First".to_string()), Health(1))).unwrap();
    assert!(world.remove_observer(observer));
    let _second = world
        .spawn((Name("Second".to_string()), Health(2)))
        .unwrap();

    assert_eq!(*events.lock().unwrap(), vec![first]);
}

#[test]
fn immediate_entity_event_observers_run_global_then_targeted_callbacks() {
    let mut world = World::empty();
    let first = world.spawn((Name("First".to_string()),)).unwrap();
    let second = world.spawn((Name("Second".to_string()),)).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));

    {
        let events = events.clone();
        world.observe_event::<DamageEvent>(move |_world, event| {
            events.lock().unwrap().push(format!("global={}", event.0));
        });
    }
    {
        let events = events.clone();
        world.observe_entity_event::<DamageEvent>(first, move |world, entity, event| {
            world.insert(entity, Marker).unwrap();
            events
                .lock()
                .unwrap()
                .push(format!("targeted={entity}:{}", event.0));
        });
    }

    world.trigger_entity_event(first, DamageEvent(7));
    world.trigger_entity_event(second, DamageEvent(3));

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "global=7".to_string(),
            format!("targeted={first}:7"),
            "global=3".to_string(),
        ]
    );
    assert_eq!(world.get::<Marker>(first), Some(&Marker));
    assert_eq!(world.get::<Marker>(second), None);
}

#[test]
fn message_reader_param_uses_a_persistent_cursor() {
    let mut world = World::empty();
    type Writer = MessageWriterParam<DamageMessage>;
    type Reader = MessageReaderParam<DamageMessage>;

    let mut writer = SystemState::<Writer>::new(&mut world).unwrap();
    writer.run(&mut world, |mut messages| {
        assert_eq!(messages.write(DamageMessage(1)).id(), 0);
        assert_eq!(messages.write(DamageMessage(2)).id(), 1);
    });

    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();
    let first_read = reader.run(&mut world, |mut messages| {
        assert_eq!(messages.unread_count(), 2);
        assert_eq!(messages.len(), 2);
        assert!(!messages.is_empty());
        messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>()
    });
    assert_eq!(first_read, vec![1, 2]);

    let second_read = reader.run(&mut world, |mut messages| {
        assert_eq!(messages.unread_count(), 0);
        assert_eq!(messages.len(), 0);
        assert!(messages.is_empty());
        messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>()
    });
    assert!(second_read.is_empty());

    assert_eq!(world.send_message(DamageMessage(3)).id(), 2);
    let third_read = reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>()
    });
    assert_eq!(third_read, vec![3]);
}

#[test]
fn message_writer_batch_preserves_order_and_ids() {
    let mut world = World::empty();
    type Writer = MessageWriterParam<DamageMessage>;
    type Reader = MessageReaderParam<DamageMessage>;

    let mut writer = SystemState::<Writer>::new(&mut world).unwrap();
    let ids = writer.run(&mut world, |mut messages| {
        messages.write_batch([DamageMessage(1), DamageMessage(2), DamageMessage(3)])
    });
    assert_eq!(
        ids.iter().map(|id| id.id()).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );

    let more_ids = writer.run(&mut world, |mut messages| {
        messages.write_batch([DamageMessage(4), DamageMessage(5)])
    });
    assert_eq!(
        more_ids.iter().map(|id| id.id()).collect::<Vec<_>>(),
        vec![3, 4]
    );

    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();
    let observed = reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(observed, vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
}

#[test]
fn message_reader_param_observes_messages_after_global_clear() {
    let mut world = World::empty();
    type Reader = MessageReaderParam<DamageMessage>;

    world.send_message(DamageMessage(1));
    world.send_message(DamageMessage(2));

    let mut reader = SystemState::<Reader>::new(&mut world).unwrap();
    let first_read = reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>()
    });
    assert_eq!(first_read, vec![1, 2]);

    world.clear_messages::<DamageMessage>();
    assert_eq!(world.send_message(DamageMessage(3)).id(), 2);

    let after_clear = reader.run(&mut world, |mut messages| {
        assert_eq!(messages.unread_count(), 1);
        messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>()
    });
    assert_eq!(after_clear, vec![3]);
}

#[test]
fn message_reader_clear_advances_only_that_reader_cursor() {
    let mut world = World::empty();
    type Reader = MessageReaderParam<DamageMessage>;

    world.send_message(DamageMessage(1));
    world.send_message(DamageMessage(2));

    let mut first_reader = SystemState::<Reader>::new(&mut world).unwrap();
    let mut second_reader = SystemState::<Reader>::new(&mut world).unwrap();

    let cleared = first_reader.run(&mut world, |mut messages| {
        assert_eq!(messages.len(), 2);
        messages.clear();
        assert!(messages.is_empty());
        messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>()
    });
    assert!(cleared.is_empty());

    let observed_by_other_reader = second_reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>()
    });
    assert_eq!(observed_by_other_reader, vec![1, 2]);
}
