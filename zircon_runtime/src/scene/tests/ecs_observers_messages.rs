use std::sync::{Arc, Mutex};

use crate::scene::components::Name;
use crate::scene::ecs::{
    Component, LifecycleEventKind, Message, MessageId, MessageReaderParam, MessageWriterParam,
    ObserverId, SystemState,
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
fn event_observer_handles_can_be_removed_before_later_triggers() {
    let mut world = World::empty();
    let target = world.spawn((Name("Target".to_string()),)).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));

    let global_observer = {
        let events = events.clone();
        world.observe_event::<DamageEvent>(move |_world, event| {
            events.lock().unwrap().push(format!("global={}", event.0));
        })
    };
    let target_observer = {
        let events = events.clone();
        world.observe_entity_event::<DamageEvent>(target, move |_world, entity, event| {
            events
                .lock()
                .unwrap()
                .push(format!("target={entity}:{}", event.0));
        })
    };

    assert!(world.remove_observer(global_observer));
    assert!(world.remove_observer(target_observer));
    assert!(!world.remove_observer(global_observer));
    world.trigger_entity_event(target, DamageEvent(9));

    assert!(events.lock().unwrap().is_empty());
}

#[test]
fn lifecycle_observer_fires_immediately_during_component_mutation() {
    let mut world = World::empty();
    let observed = Arc::new(Mutex::new(Vec::new()));

    {
        let observed = observed.clone();
        world.observe_component_lifecycle::<Health>(
            LifecycleEventKind::Add,
            move |world, event| {
                observed.lock().unwrap().push(format!(
                    "add:{}:{}",
                    event.entity(),
                    world.get::<Health>(event.entity()).is_some()
                ));
            },
        );
    }

    let entity = world
        .spawn((Name("Immediate".to_string()), Health(1)))
        .unwrap();

    assert_eq!(
        *observed.lock().unwrap(),
        vec![format!("add:{entity}:true")]
    );
}

#[test]
fn entity_event_observer_only_fires_for_target_entity() {
    let mut world = World::empty();
    let target = world.spawn((Name("Target".to_string()),)).unwrap();
    let other = world.spawn((Name("Other".to_string()),)).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));

    {
        let events = events.clone();
        world.observe_entity_event::<DamageEvent>(target, move |_world, entity, event| {
            events.lock().unwrap().push(format!("{entity}:{}", event.0));
        });
    }

    world.trigger_entity_event(other, DamageEvent(1));
    world.trigger_entity_event(target, DamageEvent(2));

    assert_eq!(*events.lock().unwrap(), vec![format!("{target}:2")]);
}

#[test]
fn observer_remove_during_dispatch_does_not_skip_or_double_fire() {
    let mut world = World::empty();
    let events = Arc::new(Mutex::new(Vec::new()));
    let target_observer = Arc::new(Mutex::new(None::<ObserverId>));

    {
        let events = events.clone();
        let target_observer = target_observer.clone();
        world.observe_event::<DamageEvent>(move |world, event| {
            if let Some(observer) = *target_observer.lock().unwrap() {
                let removed = world.remove_observer(observer);
                events
                    .lock()
                    .unwrap()
                    .push(format!("remove={}:{}", event.0, removed));
            }
        });
    }

    let removed_during_dispatch = {
        let events = events.clone();
        world.observe_event::<DamageEvent>(move |_world, event| {
            events.lock().unwrap().push(format!("target={}", event.0));
        })
    };
    *target_observer.lock().unwrap() = Some(removed_during_dispatch);

    world.trigger_event(DamageEvent(1));
    world.trigger_event(DamageEvent(2));

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "remove=1:true".to_string(),
            "target=1".to_string(),
            "remove=2:false".to_string(),
        ]
    );
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
fn observer_callback_collection_uses_exact_capacity_vectors() {
    let observer_source = include_str!("../ecs/observer.rs");

    assert!(observer_source.contains("fn lifecycle_callback_count("));
    assert!(observer_source.contains("fn event_callback_count("));
    assert!(observer_source.contains("fn entity_event_callback_count("));
    assert!(observer_source.contains("Vec::with_capacity(callback_count)"));
    assert!(observer_source.contains("callbacks.push(observer.callback.clone());"));
    let lifecycle_count_body = observer_source
        .split("fn lifecycle_callback_count(")
        .nth(1)
        .and_then(|text| text.split("fn event_callback_count(").next())
        .expect("read lifecycle callback counter body");
    let event_count_body = observer_source
        .split("fn event_callback_count(")
        .nth(1)
        .and_then(|text| text.split("fn entity_event_callback_count(").next())
        .expect("read event callback counter body");
    let entity_event_count_body = observer_source
        .split("fn entity_event_callback_count(")
        .nth(1)
        .and_then(|text| text.split("fn remove_observer_by_id<T>(").next())
        .expect("read entity event callback counter body");
    for count_body in [
        lifecycle_count_body,
        event_count_body,
        entity_event_count_body,
    ] {
        assert!(count_body.contains("let mut count = 0_usize;"));
        assert!(count_body.contains("count += 1;"));
        assert!(!count_body.contains(".filter("));
        assert!(!count_body.contains(".count()"));
    }
    assert!(!observer_source
        .contains(".map(|observer| observer.callback.clone())\n            .collect()"));
    assert!(observer_source.contains("fn remove_observer_by_id<T>("));
    assert!(observer_source.contains("Observer ids are allocated from one counter"));
    assert!(observer_source.contains("let mut index = 0_usize;"));
    assert!(observer_source.contains("while index < observers.len()"));
    assert!(observer_source.contains("observer_id(&observers[index]) == id"));
    assert!(observer_source.contains("index += 1;"));
    assert!(observer_source.contains("observers.remove(index);"));
    assert!(observer_source.contains("return true;"));
    assert!(!observer_source.contains(".position(|observer| observer_id(observer) == id)"));
    assert!(!observer_source.contains("let before = self.total_len();"));
    assert!(!observer_source.contains("self.total_len() != before"));
    assert!(!observer_source.contains(".retain(|observer| observer.id != id)"));
}

#[test]
fn event_and_message_batch_writers_preallocate_from_size_hint() {
    let events_source = include_str!("../ecs/events.rs");
    assert!(events_source.contains("let events = events.into_iter();"));
    assert!(events_source.contains("let (lower_bound, _) = events.size_hint();"));
    assert!(events_source.contains("self.next.reserve(lower_bound);"));
    assert!(events_source.contains("self.next.push(event);"));
    assert!(!events_source.contains("for event in events {\n            self.send(event);"));
    assert!(events_source.contains("std::mem::take(&mut self.current)"));
    assert!(!events_source.contains("self.current.drain(..).collect()"));

    let messages_source = include_str!("../ecs/messages.rs");
    assert!(messages_source.contains("pub fn write_batch<I>(&mut self, messages: I)"));
    assert!(messages_source.contains("let messages = messages.into_iter();"));
    assert!(messages_source.contains("let (lower_bound, _) = messages.size_hint();"));
    assert!(messages_source.contains("self.messages.reserve(lower_bound);"));
    assert!(messages_source.contains("let mut ids = Vec::with_capacity(lower_bound);"));
    assert!(messages_source.contains("ids.push(self.write(message));"));
    assert!(messages_source.contains("pub fn write_batch<T, I>(&mut self, messages: I)"));

    let system_messages_source = include_str!("../ecs/system/messages.rs");
    assert!(system_messages_source.contains("self.store.write_batch::<T, I>(messages)"));
    assert!(!system_messages_source.contains(".map(|message| self.write(message))"));
}

#[test]
fn event_and_message_type_name_lists_preallocate_from_registered_type_count() {
    let events_source = include_str!("../ecs/events.rs");
    assert!(events_source.contains("let mut names = Vec::with_capacity(self.type_names.len());"));
    assert!(events_source.contains("for name in self.type_names.values()"));
    assert!(events_source.contains("names.push(*name);"));
    assert!(!events_source.contains("self.type_names.values().copied().collect::<Vec<_>>()"));

    let messages_source = include_str!("../ecs/messages.rs");
    assert!(messages_source.contains("let mut names = Vec::with_capacity(self.type_names.len());"));
    assert!(messages_source.contains("for name in self.type_names.values()"));
    assert!(messages_source.contains("names.push(*name);"));
    assert!(!messages_source.contains("self.type_names.values().copied().collect::<Vec<_>>()"));
}

#[test]
fn message_id_debug_uses_cached_type_name_tail_branch() {
    let messages_source = include_str!("../ecs/messages.rs");

    assert_eq!(
        format!("{:?}", MessageId::<DamageMessage>::new(7)),
        "message<DamageMessage>#7"
    );
    assert!(messages_source.contains("let message_type_name = type_name::<T>();"));
    assert!(messages_source
        .contains("let message_type_label = match message_type_name.rsplit(\"::\").next()"));
    assert!(messages_source.contains("Some(label) => label,"));
    assert!(messages_source.contains("None => message_type_name,"));
    assert!(!messages_source.contains(".unwrap_or(type_name::<T>())"));
}

#[test]
fn event_and_message_cursors_use_direct_lookup_branches() {
    let events_source = include_str!("../ecs/events.rs");
    let event_unread_source = events_source
        .split("pub fn unread_count(&self, events: Option<&Events<T>>) -> usize")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn clear(&mut self, events: Option<&Events<T>>)")
                .next()
        })
        .expect("read EventCursor::unread_count body");
    let event_store_lookup_source = events_source
        .split("pub fn events<T: 'static + Send + Sync>(&self) -> Option<&Events<T>>")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn events_mut<T: 'static + Send + Sync>")
                .next()
        })
        .expect("read EventStore::events body");

    assert!(event_unread_source.contains("let Some(events) = events else"));
    assert!(event_unread_source.contains("return 0;"));
    assert!(event_unread_source.contains("if self.generation == events.generation()"));
    assert!(
        event_unread_source.contains("events.len().saturating_sub(self.cursor.min(events.len()))")
    );
    assert!(event_unread_source.contains("events.len()"));
    assert!(!event_unread_source.contains(".map(|events|"));
    assert!(!event_unread_source.contains(".unwrap_or_default()"));
    assert!(event_store_lookup_source.contains("let store = self.stores.get(&TypeId::of::<T>())?;"));
    assert!(event_store_lookup_source.contains("store.downcast_ref::<Events<T>>()"));
    assert!(
        !event_store_lookup_source.contains(".and_then(|store| store.downcast_ref::<Events<T>>())")
    );

    let messages_source = include_str!("../ecs/messages.rs");
    let message_unread_source = messages_source
        .split("pub fn unread_count(&self, messages: Option<&Messages<T>>) -> usize")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn clear(&mut self, messages: Option<&Messages<T>>)")
                .next()
        })
        .expect("read MessageCursor::unread_count body");
    let message_store_lookup_source = messages_source
        .split("pub fn messages<T>(&self) -> Option<&Messages<T>>")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn messages_mut<T>(&mut self) -> &mut Messages<T>")
                .next()
        })
        .expect("read MessageStore::messages body");

    assert!(message_unread_source.contains("let Some(messages) = messages else"));
    assert!(message_unread_source.contains("return 0;"));
    assert!(message_unread_source.contains("if self.generation == messages.generation()"));
    assert!(message_unread_source.contains(
        "messages.messages.len().saturating_sub(self.cursor.min(messages.messages.len()))"
    ));
    assert!(message_unread_source.contains("messages.messages.len()"));
    assert!(!message_unread_source.contains(".map(|messages|"));
    assert!(!message_unread_source.contains(".unwrap_or_default()"));
    assert!(
        message_store_lookup_source.contains("let store = self.stores.get(&TypeId::of::<T>())?;")
    );
    assert!(message_store_lookup_source.contains("store.downcast_ref::<Messages<T>>()"));
    assert!(!message_store_lookup_source
        .contains(".and_then(|store| store.downcast_ref::<Messages<T>>())"));
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
