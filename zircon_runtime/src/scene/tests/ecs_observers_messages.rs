use std::sync::{Arc, Mutex};

use crate::scene::components::Name;
use crate::scene::ecs::{
    Component, LifecycleEventKind, Message, MessageId, MessageReaderParam, MessageRetention,
    MessageWriterParam, ObserverId, ObserverStore, SystemStage, SystemState,
};
use crate::scene::{SceneError, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

zircon_runtime_interface::impl_retained_owner_v1!(Health { 0 });
impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

zircon_runtime_interface::impl_retained_owner_v1!(Marker {});
impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct DamageEvent(u32);

#[derive(Debug, PartialEq, Eq)]
struct DamageMessage(u32);

impl Message for DamageMessage {}

fn event_source() -> &'static str {
    concat!(
        include_str!("../ecs/events/mod.rs"),
        "\n",
        include_str!("../ecs/events/cursor.rs"),
        "\n",
        include_str!("../ecs/events/id.rs"),
        "\n",
        include_str!("../ecs/events/metrics.rs"),
        "\n",
        include_str!("../ecs/events/queue.rs"),
        "\n",
        include_str!("../ecs/events/store.rs"),
        "\n",
        include_str!("../ecs/events/subscription.rs"),
    )
}

fn message_source() -> &'static str {
    concat!(
        include_str!("../ecs/messages/mod.rs"),
        "\n",
        include_str!("../ecs/messages/cursor.rs"),
        "\n",
        include_str!("../ecs/messages/id.rs"),
        "\n",
        include_str!("../ecs/messages/queue.rs"),
        "\n",
        include_str!("../ecs/messages/store.rs"),
    )
}

fn observer_source() -> &'static str {
    concat!(
        include_str!("../ecs/observer/mod.rs"),
        "\n",
        include_str!("../ecs/observer/callback_registry.rs"),
        "\n",
        include_str!("../ecs/observer/callbacks.rs"),
        "\n",
        include_str!("../ecs/observer/entry.rs"),
        "\n",
        include_str!("../ecs/observer/id.rs"),
        "\n",
        include_str!("../ecs/observer/store.rs"),
    )
}

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
        world.observe_component_lifecycle::<Health>(
            kind,
            // SAFETY: Only the external observation log changes after optional World reads;
            // retained World payload is untouched.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_world, event| {
                    events.lock().unwrap().push(format!(
                        "{:?}:{}:{}",
                        event.kind(),
                        event.entity(),
                        event.component_id().index()
                    ));
                })
            },
        );
    }

    let entity = world
        .spawn((Name("Observed".to_string()), Health(1)))
        .unwrap();
    world.insert(entity, Health(2)).unwrap();
    world.remove::<Health>(entity).unwrap();
    world.insert(entity, Health(3)).unwrap();
    world.remove_entity(entity).unwrap();

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
            // SAFETY: Only the external observation log changes after optional World reads;
            // retained World payload is untouched.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_world, event| {
                    events.lock().unwrap().push(event.entity());
                })
            },
        )
    };

    let first = world.spawn((Name("First".to_string()), Health(1))).unwrap();
    assert!(world.remove_observer(observer).is_ok());
    let _second = world
        .spawn((Name("Second".to_string()), Health(2)))
        .unwrap();

    assert_eq!(*events.lock().unwrap(), vec![first]);
}

#[test]
fn event_observer_handles_can_be_removed_before_later_triggers() {
    let mut world = World::empty();
    let target = world.spawn((Name("Target".to_string()),)).unwrap();
    let events = Arc::new(crate::tests::prelude::RetainedObserverLogV1::new());

    let global_observer = {
        let events = events.clone();
        world.observe_event::<DamageEvent>(move |_world, event| {
            events.lock().unwrap().push(format!("global={}", event.0));
        })
    };
    let target_observer = {
        let events = events.clone();
        world.observe_entity_event::<DamageEvent>(
            target,
            crate::core::framework::retained_memory::RetainedEntityObserverV1::new(
                events,
                |events, _world, entity, event| {
                    events
                        .lock()
                        .unwrap()
                        .push(format!("target={entity}:{}", event.0));
                },
            ),
        )
    };

    assert!(world.remove_observer(global_observer).is_ok());
    assert!(world.remove_observer(target_observer).is_ok());
    assert_eq!(
        world.remove_observer(global_observer),
        Err(SceneError::MissingObserver {
            observer: global_observer,
        })
    );
    world.trigger_entity_event(target, DamageEvent(9));

    assert!(events.lock().unwrap().is_empty());
}

#[test]
fn removing_an_already_removed_observer_returns_a_typed_scene_error() {
    let mut world = World::empty();
    let observer = world.observe_event::<DamageEvent>(|_world, _event| {});

    world.remove_observer(observer).unwrap();

    assert_eq!(
        world.remove_observer(observer),
        Err(SceneError::MissingObserver { observer })
    );
}

#[test]
fn observer_store_removal_reports_a_missing_handle_with_scene_error() {
    let mut observers = ObserverStore::default();
    let observer = ObserverId::new(7);

    assert_eq!(
        observers.remove(observer),
        Err(SceneError::MissingObserver { observer })
    );
}

#[test]
fn lifecycle_observer_fires_immediately_during_component_mutation() {
    let mut world = World::empty();
    let observed = Arc::new(Mutex::new(Vec::new()));

    {
        let observed = observed.clone();
        world.observe_component_lifecycle::<Health>(
            LifecycleEventKind::Add,
            // SAFETY: Only the external observation log changes after optional World reads;
            // retained World payload is untouched.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, event| {
                    observed.lock().unwrap().push(format!(
                        "add:{}:{}",
                        event.entity(),
                        world.get::<Health>(event.entity()).is_some()
                    ));
                })
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
    let events = Arc::new(crate::tests::prelude::RetainedObserverLogV1::new());

    {
        let events = events.clone();
        world.observe_entity_event::<DamageEvent>(
            target,
            crate::core::framework::retained_memory::RetainedEntityObserverV1::new(
                events,
                |events, _world, entity, event| {
                    events.lock().unwrap().push(format!("{entity}:{}", event.0));
                },
            ),
        );
    }

    world.trigger_entity_event(other, DamageEvent(1));
    world.trigger_entity_event(target, DamageEvent(2));

    assert_eq!(*events.lock().unwrap(), vec![format!("{target}:2")]);
}

#[test]
fn entity_event_observers_are_removed_with_their_target_entity() {
    let mut world = World::empty();
    let target = world.spawn((Name("Target".to_string()),)).unwrap();
    let events = Arc::new(crate::tests::prelude::RetainedObserverLogV1::new());

    {
        let events = events.clone();
        world.observe_entity_event::<DamageEvent>(
            target,
            crate::core::framework::retained_memory::RetainedEntityObserverV1::new(
                events,
                |events, _world, entity, event| {
                    events.lock().unwrap().push(format!("{entity}:{}", event.0));
                },
            ),
        );
    }

    world.remove_entity(target).unwrap();
    world.trigger_entity_event(target, DamageEvent(1));

    assert!(events.lock().unwrap().is_empty());
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
                let removed = world.remove_observer(observer).is_ok();
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
fn observer_registration_during_dispatch_is_visible_on_next_trigger() {
    let mut world = World::empty();
    let events = Arc::new(Mutex::new(Vec::new()));
    let added_observer = Arc::new(Mutex::new(None::<ObserverId>));

    {
        let events = events.clone();
        let added_observer = added_observer.clone();
        world.observe_event::<DamageEvent>(move |world, event| {
            events.lock().unwrap().push(format!("first={}", event.0));
            let should_register = added_observer.lock().unwrap().is_none();
            if should_register {
                let events = events.clone();
                let observer = world.observe_event::<DamageEvent>(move |_world, event| {
                    events.lock().unwrap().push(format!("added={}", event.0));
                });
                *added_observer.lock().unwrap() = Some(observer);
            }
        });
    }

    world.trigger_event(DamageEvent(1));
    world.trigger_event(DamageEvent(2));

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "first=1".to_string(),
            "first=2".to_string(),
            "added=2".to_string(),
        ]
    );
}

#[test]
fn immediate_entity_event_observers_run_global_then_targeted_callbacks() {
    let mut world = World::empty();
    let first = world.spawn((Name("First".to_string()),)).unwrap();
    let second = world.spawn((Name("Second".to_string()),)).unwrap();
    let events = Arc::new(crate::tests::prelude::RetainedObserverLogV1::new());

    {
        let events = events.clone();
        world.observe_event::<DamageEvent>(move |_world, event| {
            events.lock().unwrap().push(format!("global={}", event.0));
        });
    }
    {
        let events = events.clone();
        world.observe_entity_event::<DamageEvent>(
            first,
            crate::core::framework::retained_memory::RetainedEntityObserverV1::new(
                events,
                |events, world, entity, event| {
                    world.insert(entity, Marker).unwrap();
                    events
                        .lock()
                        .unwrap()
                        .push(format!("targeted={entity}:{}", event.0));
                },
            ),
        );
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
fn observer_dispatch_uses_indexed_immutable_buckets() {
    let observer_source = observer_source();

    assert!(observer_source.contains("HashMap<LifecycleObserverKey, Arc<Vec<LifecycleObserver>>>"));
    assert!(observer_source.contains("HashMap<TypeId, Arc<Vec<EventObserver>>>"));
    assert!(
        observer_source.contains("HashMap<EntityEventObserverKey, Arc<Vec<EntityEventObserver>>>")
    );
    assert!(observer_source.contains("observer_locations: HashMap<ObserverId, ObserverBucket>"));
    assert!(observer_source
        .contains("entity_event_types_by_entity: HashMap<EntityId, HashSet<TypeId>>"));
    assert!(observer_source.contains("pub(crate) struct LifecycleCallbackBucket"));
    assert!(observer_source.contains("pub(crate) struct EventCallbackBucket"));
    assert!(observer_source.contains("pub(crate) struct EntityEventCallbackBucket"));
    assert!(observer_source.contains("pub fn remove(&mut self, id: ObserverId) -> SceneResult<()>"));
    assert!(observer_source.contains("fn insert_observer_into_bucket<T>"));
    assert!(observer_source.contains("let observers = Arc::make_mut(bucket)"));
    assert!(observer_source.contains("observers.push(observer)"));
    assert!(observer_source.contains("observers.remove(index)"));
    assert!(observer_source.contains("pub(crate) fn remove_entity_observers"));
    assert!(!observer_source.contains("lifecycle_observers: Vec<"));
    assert!(!observer_source.contains("event_observers: Vec<"));
    assert!(!observer_source.contains("entity_event_observers: Vec<"));
    assert!(!observer_source.contains("callback_count"));
    assert!(!observer_source.contains("callbacks.push(observer.callback.clone())"));
    assert!(!observer_source.contains("fn remove_observer_by_id<T>("));
    assert!(!observer_source.contains("pub fn remove(&mut self, id: ObserverId) -> bool"));
    assert!(!observer_source.contains("fn remove_observer_from_bucket<T>("));
    assert!(!observer_source.contains("while index < bucket.len()"));
    assert!(observer_source.contains("Fn(&mut World, &ComponentLifecycleEvent) + Send + Sync"));
    assert!(
        !observer_source.contains("event.clone()"),
        "lifecycle fanout must borrow one shared event payload"
    );
}

#[test]
fn event_and_message_batch_writers_preallocate_from_size_hint() {
    let events_source = event_source();
    let events_batch_source = include_str!("../ecs/events/queue.rs")
        .split_once("pub fn send_batch<I>")
        .expect("event queue batch method")
        .1
        .split_once("\n    pub fn update")
        .expect("event queue batch method boundary")
        .0;
    assert!(events_batch_source.contains("self.next.extend(events);"));
    assert!(events_batch_source.contains("let before = self.next.len();"));
    assert!(events_batch_source.contains("let written = self.next.len() - before;"));
    assert!(
        events_batch_source.contains("if written > 0 {\n            self.record_next_queue_len();")
    );
    assert!(!events_batch_source.contains("self.send(event)"));
    assert!(events_source.contains("self.next.push(event);"));
    assert!(events_source.contains("std::mem::take(&mut self.current)"));
    assert!(!events_source.contains("self.current.drain(..).collect()"));

    let messages_queue_source = include_str!("../ecs/messages/queue.rs");
    assert!(messages_queue_source.contains("pub fn write_batch<I>(&mut self, messages: I)"));
    let messages_batch_source = messages_queue_source
        .split_once("pub(super) fn write_batch_at_frame<I>")
        .expect("message queue frame-aware batch method")
        .1
        .split_once("\n    pub fn iter")
        .expect("message queue batch method boundary")
        .0;
    assert!(messages_batch_source.contains("let messages = messages.into_iter();"));
    assert!(messages_batch_source.contains("let (lower_bound, _) = messages.size_hint();"));
    assert!(messages_batch_source.contains("let mut ids = Vec::with_capacity(lower_bound);"));
    assert!(messages_batch_source.contains("ids.push(self.write_at_frame(message, frame));"));
    let messages_store_batch_source = include_str!("../ecs/messages/store.rs")
        .split_once("pub fn write_batch<T, I>")
        .expect("message store batch method")
        .1
        .split_once("\n    pub fn clear")
        .expect("message store batch method boundary")
        .0;
    assert!(messages_store_batch_source.contains("let frame = self.frame;"));
    assert!(messages_store_batch_source.contains(".write_batch_at_frame(messages, frame)"));

    let system_messages_source = include_str!("../ecs/system/messages.rs");
    assert!(system_messages_source.contains("self.channel.write_batch(messages)"));
    let message_grant_source = include_str!("../ecs/messages/store/writer_grant.rs");
    assert!(
        message_grant_source.contains("self.messages_mut().write_batch_at_frame(messages, frame)")
    );
    assert!(!system_messages_source.contains(".map(|message| self.write(message))"));
}

#[test]
fn event_and_message_type_name_lists_preallocate_from_registered_type_count() {
    let events_source = event_source();
    assert!(events_source.contains("let mut names = Vec::with_capacity(self.channels.len());"));
    assert!(events_source.contains("for channel in &self.channels"));
    assert!(events_source.contains("names.push(channel.type_name);"));
    assert!(!events_source.contains("self.type_names.values().copied().collect::<Vec<_>>()"));

    let messages_source = message_source();
    assert!(
        messages_source.contains("let mut names = Vec::with_capacity(activity.type_names.len());")
    );
    assert!(messages_source.contains("for name in activity.type_names.values()"));
    assert!(messages_source.contains("names.push(*name);"));
    assert!(!messages_source.contains("self.type_names.values().copied().collect::<Vec<_>>()"));
}

#[test]
fn message_id_debug_uses_cached_type_name_tail_branch() {
    let messages_source = message_source();

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
    let events_source = event_source();
    let event_unread_source = events_source
        .split("pub fn unread_count(&self, events: Option<&Events<T>>) -> usize")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn clear(&mut self, events: Option<&Events<T>>)")
                .next()
        })
        .expect("read EventCursor::unread_count body");
    let event_store_lookup_source = events_source
        .split("pub fn events<T: Event>(&self) -> Option<&Events<T>>")
        .nth(1)
        .and_then(|text| text.split("pub fn events_by_id<T: Event>").next())
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
    assert!(event_store_lookup_source.contains("let event_type_id = self.event_type_id::<T>()?;"));
    assert!(event_store_lookup_source.contains("self.events_by_id(event_type_id)"));
    assert!(
        events_source.contains("pub fn events_by_id<T: Event>(&self, event_type_id: EventTypeId)")
    );
    assert!(events_source.contains("let channel = self.channel(event_type_id)?;"));
    assert!(events_source.contains(".downcast_ref::<OwnedChannel<Events<T>>>()"));
    assert!(events_source.contains(".map(OwnedChannel::get)"));
    assert!(
        !event_store_lookup_source.contains(".and_then(|store| store.downcast_ref::<Events<T>>())")
    );

    let messages_source = message_source();
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
    assert!(message_unread_source.contains("messages.read_window_start(self.next_id)"));
    assert!(message_unread_source.contains("messages.messages.len()"));
    assert!(!message_unread_source.contains("self.cursor"));
    assert!(!message_unread_source.contains(".map(|messages|"));
    assert!(!message_unread_source.contains(".unwrap_or_default()"));
    assert!(
        message_store_lookup_source.contains("let store = self.stores.get(&TypeId::of::<T>())?;")
    );
    assert!(message_store_lookup_source
        .contains(".downcast_ref::<OwnedChannel<Option<Messages<T>>>>()?"));
    assert!(message_store_lookup_source.contains(".get()"));
    assert!(message_store_lookup_source.contains(".as_ref()"));
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

    world.clear_messages::<DamageMessage>();
    world.configure_message_retention::<DamageMessage>(MessageRetention::new(8, usize::MAX, 1));
    world.run_internal_scene_systems_for_stage(SystemStage::First);
    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 0);

    let retained_ids = writer.run(&mut world, |mut messages| {
        messages.write_batch([DamageMessage(6), DamageMessage(7)])
    });
    assert_eq!(
        retained_ids.iter().map(|id| id.id()).collect::<Vec<_>>(),
        vec![5, 6]
    );
    let before_empty_batch = world.message_retention_metrics::<DamageMessage>().unwrap();
    let empty_ids = writer.run(&mut world, |mut messages| {
        messages.write_batch(std::iter::empty::<DamageMessage>())
    });
    assert!(empty_ids.is_empty());
    assert_eq!(
        world.message_retention_metrics::<DamageMessage>().unwrap(),
        before_empty_batch
    );

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 1);
    let at_age_limit = world.message_retention_metrics::<DamageMessage>().unwrap();
    assert_eq!(at_age_limit.retained_entries, 2);
    assert_eq!(
        at_age_limit.retained_bytes,
        2 * std::mem::size_of::<DamageMessage>()
    );
    assert_eq!(at_age_limit.age_dropped_entries, 0);
    assert_eq!(
        world
            .messages::<DamageMessage>()
            .unwrap()
            .iter()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>(),
        vec![(5, 6), (6, 7)]
    );

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 1);
    let retired = world.message_retention_metrics::<DamageMessage>().unwrap();
    assert_eq!(retired.retained_entries, 0);
    assert_eq!(retired.retained_bytes, 0);
    assert_eq!(retired.age_dropped_entries, 2);
    assert_eq!(
        retired.age_dropped_bytes,
        (2 * std::mem::size_of::<DamageMessage>()) as u64
    );
    assert_eq!(retired.budget_dropped_entries, 0);
    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 0);
    let next_ids = writer.run(&mut world, |mut messages| {
        messages.write_batch([DamageMessage(8)])
    });
    assert_eq!(
        next_ids.iter().map(|id| id.id()).collect::<Vec<_>>(),
        vec![7]
    );
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

mod public_lifecycle_owner {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use zircon_runtime_interface::world_sync::LifecycleTargetChange;

    use crate::scene::components::{Hierarchy, Mobility};
    use crate::scene::ecs::{
        DeferredCommandError, DeferredCommandOperation, DeferredCommandTarget,
        RemovedComponentsParam, Resource, StorageType,
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct OwnerTable(u32);
    zircon_runtime_interface::impl_retained_owner_v1!(OwnerTable { 0 });
    impl Component for OwnerTable {}

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct OwnerSparse(u32);
    zircon_runtime_interface::impl_retained_owner_v1!(OwnerSparse { 0 });
    impl Component for OwnerSparse {
        const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct OwnerResource(u32);
    impl Resource for OwnerResource {}

    fn fixture(table: u32, sparse: u32) -> (World, crate::scene::EntityId) {
        let mut world = World::empty();
        world.component_id::<OwnerTable>();
        world.component_id::<OwnerSparse>();
        let entity = world
            .spawn((OwnerTable(table), OwnerSparse(sparse)))
            .unwrap();
        (world, entity)
    }

    fn assert_replacement(
        world: &mut World,
        entity: crate::scene::EntityId,
        generation: u64,
        fresh_calls: &AtomicUsize,
    ) {
        assert!(world.contains_entity(entity));
        assert_eq!(world.get::<OwnerTable>(entity), Some(&OwnerTable(71)));
        assert_eq!(world.get::<OwnerSparse>(entity), Some(&OwnerSparse(72)));
        assert_eq!(
            world.get_resource::<OwnerResource>(),
            Some(&OwnerResource(73))
        );
        assert_eq!(world.world_generation(), generation);
        let errors = world.take_deferred_command_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].error(),
            &SceneError::Message("fresh diagnostics".into())
        );
        let mut removed = SystemState::<RemovedComponentsParam<OwnerTable>>::new(world).unwrap();
        assert!(removed
            .run(world, |mut removed| removed.read().collect::<Vec<_>>())
            .is_empty());
        assert!(world.has_deferred_commands());
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
        assert!(world.apply_deferred().is_success());
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
    }

    fn replacement_fixture(
        original: &World,
        entity: crate::scene::EntityId,
        fresh_calls: Arc<AtomicUsize>,
        leaked: Arc<AtomicUsize>,
    ) -> World {
        let (mut replacement, fresh_entity) = fixture(71, 72);
        assert_eq!(fresh_entity, entity);
        assert_eq!(
            original.registered_component_id::<OwnerTable>(),
            replacement.registered_component_id::<OwnerTable>()
        );
        assert_eq!(
            original.registered_component_id::<OwnerSparse>(),
            replacement.registered_component_id::<OwnerSparse>()
        );
        assert!(!original
            .owner_identity()
            .is_same_world(replacement.owner_identity()));
        replacement.insert_resource(OwnerResource(73));
        replacement.record_deferred_command_error(DeferredCommandError::new(
            DeferredCommandOperation::Remove,
            DeferredCommandTarget::Resolved(entity),
            SceneError::Message("fresh diagnostics".into()),
        ));
        for kind in [
            LifecycleEventKind::Remove,
            LifecycleEventKind::Despawn,
            LifecycleEventKind::Insert,
        ] {
            let leaked_table = Arc::clone(&leaked);
            replacement.observe_component_lifecycle::<OwnerTable>(
                kind,
                // SAFETY: Only an external atomic counter changes; retained World payload and
                // dynamic JSON are untouched.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                        leaked_table.fetch_add(1, Ordering::SeqCst);
                    })
                },
            );
            let leaked_sparse = Arc::clone(&leaked);
            replacement.observe_component_lifecycle::<OwnerSparse>(
                kind,
                // SAFETY: Only an external atomic counter changes; retained World payload and
                // dynamic JSON are untouched.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                        leaked_sparse.fetch_add(1, Ordering::SeqCst);
                    })
                },
            );
        }
        replacement.commands().queue_fn(move |_| {
            fresh_calls.fetch_add(1, Ordering::SeqCst);
        });
        replacement
    }

    #[test]
    fn public_remove_entity_cancels_world_replacement_at_remove_and_despawn_with_colliding_ids() {
        for retain_schedule in [false, true] {
            for boundary in [LifecycleEventKind::Remove, LifecycleEventKind::Despawn] {
                let (mut world, entity) = fixture(1, 2);
                let original_plan = world.schedule().stage_plan();
                let fresh_calls = Arc::new(AtomicUsize::new(0));
                let leaked = Arc::new(AtomicUsize::new(0));
                let replacement = replacement_fixture(
                    &world,
                    entity,
                    Arc::clone(&fresh_calls),
                    Arc::clone(&leaked),
                );
                let generation = replacement.world_generation();
                let replacement = Arc::new(Mutex::new(Some(replacement)));
                let swap = Arc::clone(&replacement);
                world.observe_component_lifecycle::<OwnerTable>(
                    boundary,
                    // SAFETY: This retires the current World or schedule. Remaining work is
                    // counter-only scheduling; replacement history is never adopted.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            move |world, _| {
                                let mut fresh = swap.lock().unwrap().take().unwrap();
                                if retain_schedule {
                                    *fresh.schedule_mut() = std::mem::take(world.schedule_mut());
                                    assert!(Arc::ptr_eq(
                                        &original_plan,
                                        &fresh.schedule().stage_plan()
                                    ));
                                }
                                *world = fresh;
                            },
                        )
                    },
                );
                assert!(matches!(
                    world.remove_entity(entity),
                    Err(SceneError::LifecycleTargetReplaced {
                        operation: "remove entity",
                        target: LifecycleTargetChange::World,
                    })
                ));
                assert_eq!(leaked.load(Ordering::SeqCst), 0);
                assert_replacement(&mut world, entity, generation, &fresh_calls);
            }
        }
    }

    #[test]
    fn public_typed_remove_cancels_table_and_sparse_world_replacement_before_storage_mutation() {
        for sparse in [false, true] {
            for retain_schedule in [false, true] {
                let (mut world, entity) = fixture(1, 2);
                let plan = world.schedule().stage_plan();
                let fresh_calls = Arc::new(AtomicUsize::new(0));
                let leaked = Arc::new(AtomicUsize::new(0));
                let replacement = replacement_fixture(
                    &world,
                    entity,
                    Arc::clone(&fresh_calls),
                    Arc::clone(&leaked),
                );
                let generation = replacement.world_generation();
                let replacement = Arc::new(Mutex::new(Some(replacement)));
                let swap =
                    move |world: &mut World, _: &crate::scene::ecs::ComponentLifecycleEvent| {
                        let mut fresh = replacement.lock().unwrap().take().unwrap();
                        if retain_schedule {
                            *fresh.schedule_mut() = std::mem::take(world.schedule_mut());
                            assert!(Arc::ptr_eq(&plan, &fresh.schedule().stage_plan()));
                        }
                        *world = fresh;
                    };
                let result = if sparse {
                    world.observe_component_lifecycle::<OwnerSparse>(
                        LifecycleEventKind::Remove,
                        // SAFETY: This retires the current World or schedule. Remaining work is
                        // counter-only scheduling; replacement history is never adopted.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                swap,
                            )
                        },
                    );
                    world.remove::<OwnerSparse>(entity).map(|_| ())
                } else {
                    world.observe_component_lifecycle::<OwnerTable>(
                        LifecycleEventKind::Remove,
                        // SAFETY: This retires the current World or schedule. Remaining work is
                        // counter-only scheduling; replacement history is never adopted.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                swap,
                            )
                        },
                    );
                    world.remove::<OwnerTable>(entity).map(|_| ())
                };
                assert!(matches!(
                    result,
                    Err(SceneError::LifecycleTargetReplaced {
                        operation: "remove component",
                        target: LifecycleTargetChange::World,
                    })
                ));
                assert_eq!(leaked.load(Ordering::SeqCst), 0);
                assert_replacement(&mut world, entity, generation, &fresh_calls);
            }
        }
    }

    #[test]
    fn public_removal_cancels_schedule_only_replacement_and_preserves_rows_and_fresh_queue() {
        for operation in 0..3 {
            let (mut world, entity) = fixture(1, 2);
            let owner = world.owner_identity().clone();
            let fresh_calls = Arc::new(AtomicUsize::new(0));
            let observed = Arc::clone(&fresh_calls);
            world.observe_component_lifecycle::<OwnerTable>(
                LifecycleEventKind::Remove,
                // SAFETY: After World or schedule retirement, only fixed resources and counter-
                // only commands change; no replacement history is adopted.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, _| {
                        *world.schedule_mut() = Default::default();
                        let observed = Arc::clone(&observed);
                        world.commands().queue_fn(move |_| {
                            observed.fetch_add(1, Ordering::SeqCst);
                        });
                    })
                },
            );
            if operation == 2 {
                let observed = Arc::clone(&fresh_calls);
                world.observe_component_lifecycle::<OwnerSparse>(
                    LifecycleEventKind::Remove,
                    // SAFETY: After World or schedule retirement, only fixed resources and
                    // counter-only commands change; no replacement history is adopted.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            move |world, _| {
                                *world.schedule_mut() = Default::default();
                                let observed = Arc::clone(&observed);
                                world.commands().queue_fn(move |_| {
                                    observed.fetch_add(1, Ordering::SeqCst);
                                });
                            },
                        )
                    },
                );
            }
            let result = match operation {
                0 => world.remove_entity(entity),
                1 => world.remove::<OwnerTable>(entity).map(|_| ()),
                2 => world.remove::<OwnerSparse>(entity).map(|_| ()),
                _ => unreachable!(),
            };
            assert!(matches!(
                result,
                Err(SceneError::LifecycleTargetReplaced {
                    target: LifecycleTargetChange::Schedule,
                    ..
                })
            ));
            assert!(owner.is_same_world(world.owner_identity()));
            assert_eq!(world.get::<OwnerTable>(entity), Some(&OwnerTable(1)));
            assert_eq!(world.get::<OwnerSparse>(entity), Some(&OwnerSparse(2)));
            assert!(world.has_deferred_commands());
            assert!(world.apply_deferred().is_success());
            assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn orphan_hierarchy_replacement_cancels_helper_insert_and_parent_removal() {
        let mut world = World::empty();
        let parent = world.spawn(()).unwrap();
        let first = world.spawn(()).unwrap();
        let second = world.spawn(()).unwrap();
        world.set_mobility(first, Mobility::Dynamic).unwrap();
        world.set_mobility(second, Mobility::Dynamic).unwrap();
        world.set_parent_checked(first, Some(parent)).unwrap();
        world.set_parent_checked(second, Some(parent)).unwrap();
        let mut fresh = World::empty();
        assert_eq!(fresh.spawn(()).unwrap(), parent);
        assert_eq!(fresh.spawn(()).unwrap(), first);
        assert_eq!(fresh.spawn(()).unwrap(), second);
        fresh.set_mobility(first, Mobility::Dynamic).unwrap();
        fresh.set_mobility(second, Mobility::Dynamic).unwrap();
        fresh.set_parent_checked(first, Some(parent)).unwrap();
        fresh.set_parent_checked(second, Some(parent)).unwrap();
        assert_eq!(
            world.registered_component_id::<Hierarchy>(),
            fresh.registered_component_id::<Hierarchy>()
        );
        let generation = fresh.world_generation();
        let leaked = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&leaked);
        fresh.observe_component_lifecycle::<Hierarchy>(
            LifecycleEventKind::Insert,
            // SAFETY: Only an external atomic counter changes; retained World payload and
            // dynamic JSON are untouched.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                    observed.fetch_add(1, Ordering::SeqCst);
                })
            },
        );
        let fresh = Arc::new(Mutex::new(Some(fresh)));
        world.observe_component_lifecycle::<Hierarchy>(
            LifecycleEventKind::Replace,
            // SAFETY: This retires the current World or schedule. Remaining work is counter-
            // only scheduling; replacement history is never adopted.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, _| {
                    *world = fresh.lock().unwrap().take().unwrap();
                })
            },
        );
        assert!(matches!(
            world.remove_entity(parent),
            Err(SceneError::LifecycleTargetReplaced {
                operation: "insert component",
                target: LifecycleTargetChange::World,
            })
        ));
        assert_eq!(leaked.load(Ordering::SeqCst), 0);
        assert!(world.contains_entity(parent));
        assert_eq!(world.parent_of(first), Some(parent));
        assert_eq!(world.parent_of(second), Some(parent));
        assert_eq!(world.world_generation(), generation);
    }

    #[test]
    fn public_insert_cancels_first_and_final_callback_replacement() {
        for was_present in [false, true] {
            for final_phase in [false, true] {
                for schedule_only in [false, true] {
                    let mut world = World::empty();
                    world.component_id::<OwnerTable>();
                    let entity = world.spawn(()).unwrap();
                    if was_present {
                        world.insert(entity, OwnerTable(1)).unwrap();
                    }
                    let leaked = Arc::new(AtomicUsize::new(0));
                    let mut fresh = World::empty();
                    fresh.component_id::<OwnerTable>();
                    assert_eq!(fresh.spawn(()).unwrap(), entity);
                    fresh.insert(entity, OwnerTable(71)).unwrap();
                    let observed = Arc::clone(&leaked);
                    fresh.observe_component_lifecycle::<OwnerTable>(
                        LifecycleEventKind::Insert,
                        // SAFETY: Only an external atomic counter changes; retained World
                        // payload and dynamic JSON are untouched.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                move |_, _| {
                                    observed.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                        },
                    );
                    let fresh = Arc::new(Mutex::new(Some(fresh)));
                    let boundary = if final_phase {
                        LifecycleEventKind::Insert
                    } else if was_present {
                        LifecycleEventKind::Replace
                    } else {
                        LifecycleEventKind::Add
                    };
                    world.observe_component_lifecycle::<OwnerTable>(
                        boundary,
                        // SAFETY: This retires the current World or schedule. Remaining work is
                        // counter-only scheduling; replacement history is never adopted.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                move |world, _| {
                                    if schedule_only {
                                        *world.schedule_mut() = Default::default();
                                    } else {
                                        *world = fresh.lock().unwrap().take().unwrap();
                                    }
                                },
                            )
                        },
                    );
                    assert!(matches!(
                        world.insert(entity, OwnerTable(2)),
                        Err(SceneError::LifecycleTargetReplaced {
                            operation: "insert component",
                            target,
                        }) if target == (if schedule_only { LifecycleTargetChange::Schedule } else { LifecycleTargetChange::World })
                    ));
                    assert_eq!(leaked.load(Ordering::SeqCst), 0);
                    assert_eq!(
                        world.get::<OwnerTable>(entity),
                        Some(&OwnerTable(if schedule_only { 2 } else { 71 }))
                    );
                }
            }
        }
    }

    #[test]
    fn public_entity_removal_keeps_all_component_callbacks_on_one_plan_during_valid_registration() {
        let (mut world, entity) = fixture(1, 2);
        let original = world.schedule().stage_plan();
        let callbacks = Arc::new(AtomicUsize::new(0));
        let systems = Arc::new(AtomicUsize::new(0));
        for kind in [LifecycleEventKind::Remove, LifecycleEventKind::Despawn] {
            let callback = {
                let callbacks = Arc::clone(&callbacks);
                let systems = Arc::clone(&systems);
                let original = Arc::clone(&original);
                move |world: &mut World, _: &crate::scene::ecs::ComponentLifecycleEvent| {
                    assert_eq!(world.get::<OwnerTable>(entity), Some(&OwnerTable(1)));
                    assert_eq!(world.get::<OwnerSparse>(entity), Some(&OwnerSparse(2)));
                    assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                    if callbacks.fetch_add(1, Ordering::SeqCst) == 0 {
                        let systems = Arc::clone(&systems);
                        world
                            .register_native_system::<(), _>(
                                "tests.public-remove.pending",
                                SystemStage::Update,
                                0,
                                move |()| {
                                    systems.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                            .unwrap();
                    }
                    assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                }
            };
            let callback_sparse = callback.clone();
            world.observe_component_lifecycle::<OwnerTable>(
                kind,
                // SAFETY: The local callback only observes and registers a system that
                // increments an external counter; no history payload grows.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(callback)
                },
            );
            world.observe_component_lifecycle::<OwnerSparse>(
                kind,
                // SAFETY: The local callback only observes and registers a system that
                // increments an external counter; no history payload grows.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(callback_sparse)
                },
            );
        }
        world.remove_entity(entity).unwrap();
        assert_eq!(callbacks.load(Ordering::SeqCst), 4);
        assert!(!world.contains_entity(entity));
        assert!(!Arc::ptr_eq(&original, &world.schedule().stage_plan()));
        assert!(world.run_native_scene_system("tests.public-remove.pending"));
        assert_eq!(systems.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn public_insert_and_typed_remove_keep_valid_registration_pending_through_all_callbacks() {
        for sparse in [false, true] {
            let (mut world, entity) = fixture(1, 2);
            let original = world.schedule().stage_plan();
            let callbacks = Arc::new(AtomicUsize::new(0));
            let systems = Arc::new(AtomicUsize::new(0));
            let callback = {
                let callbacks = Arc::clone(&callbacks);
                let systems = Arc::clone(&systems);
                let original = Arc::clone(&original);
                move |world: &mut World, _: &crate::scene::ecs::ComponentLifecycleEvent| {
                    assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                    if callbacks.fetch_add(1, Ordering::SeqCst) == 0 {
                        let systems = Arc::clone(&systems);
                        world
                            .register_native_system::<(), _>(
                                "tests.public-typed.pending",
                                SystemStage::Update,
                                0,
                                move |()| {
                                    systems.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                            .unwrap();
                    }
                    assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                }
            };
            if sparse {
                world.observe_component_lifecycle::<OwnerSparse>(
                    LifecycleEventKind::Remove,
                    // SAFETY: The local callback only observes and registers a system that
                    // increments an external counter; no history payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(callback.clone())
                    },
                );
                world.observe_component_lifecycle::<OwnerSparse>(
                    LifecycleEventKind::Remove,
                    // SAFETY: The local callback only observes and registers a system that
                    // increments an external counter; no history payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(callback)
                    },
                );
                assert_eq!(
                    world.remove::<OwnerSparse>(entity).unwrap(),
                    Some(OwnerSparse(2))
                );
            } else {
                world.observe_component_lifecycle::<OwnerTable>(
                    LifecycleEventKind::Remove,
                    // SAFETY: The local callback only observes and registers a system that
                    // increments an external counter; no history payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(callback.clone())
                    },
                );
                world.observe_component_lifecycle::<OwnerTable>(
                    LifecycleEventKind::Remove,
                    // SAFETY: The local callback only observes and registers a system that
                    // increments an external counter; no history payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(callback)
                    },
                );
                assert_eq!(
                    world.remove::<OwnerTable>(entity).unwrap(),
                    Some(OwnerTable(1))
                );
            }
            assert_eq!(callbacks.load(Ordering::SeqCst), 2);
            assert!(!Arc::ptr_eq(&original, &world.schedule().stage_plan()));
            assert!(world.run_native_scene_system("tests.public-typed.pending"));
            assert_eq!(systems.load(Ordering::SeqCst), 1);
        }
        for was_present in [false, true] {
            let mut world = World::empty();
            let entity = world.spawn(()).unwrap();
            if was_present {
                world.insert(entity, OwnerTable(1)).unwrap();
            }
            let original = world.schedule().stage_plan();
            let callbacks = Arc::new(AtomicUsize::new(0));
            let systems = Arc::new(AtomicUsize::new(0));
            for kind in [
                if was_present {
                    LifecycleEventKind::Replace
                } else {
                    LifecycleEventKind::Add
                },
                LifecycleEventKind::Insert,
            ] {
                let original = Arc::clone(&original);
                let callbacks = Arc::clone(&callbacks);
                let systems = Arc::clone(&systems);
                world.observe_component_lifecycle::<OwnerTable>(
                    kind,
                    // SAFETY: The local callback only observes and registers a system that
                    // increments an external counter; no history payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            move |world, _| {
                                assert_eq!(world.get::<OwnerTable>(entity), Some(&OwnerTable(2)));
                                assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                                if callbacks.fetch_add(1, Ordering::SeqCst) == 0 {
                                    let systems = Arc::clone(&systems);
                                    world
                                        .register_native_system::<(), _>(
                                            "tests.public-insert.pending",
                                            SystemStage::Update,
                                            0,
                                            move |()| {
                                                systems.fetch_add(1, Ordering::SeqCst);
                                            },
                                        )
                                        .unwrap();
                                }
                                assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                            },
                        )
                    },
                );
            }
            world.insert(entity, OwnerTable(2)).unwrap();
            assert_eq!(callbacks.load(Ordering::SeqCst), 2);
            assert!(!Arc::ptr_eq(&original, &world.schedule().stage_plan()));
            assert!(world.run_native_scene_system("tests.public-insert.pending"));
            assert_eq!(systems.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn public_remove_panic_releases_retained_original_execution_without_touching_replacement() {
        let (mut world, entity) = fixture(1, 2);
        let retained = Arc::new(Mutex::new(None));
        let saved = Arc::clone(&retained);
        let original = world.schedule().stage_plan();
        let systems = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&systems);
        world.observe_component_lifecycle::<OwnerTable>(
            LifecycleEventKind::Remove,
            // SAFETY: This schedules only an external counter, retires the owner, and
            // propagates the original panic; no history payload grows.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, _| {
                    let observed = Arc::clone(&observed);
                    world
                        .register_native_system::<(), _>(
                            "tests.public-remove.panic-pending",
                            SystemStage::Update,
                            0,
                            move |()| {
                                observed.fetch_add(1, Ordering::SeqCst);
                            },
                        )
                        .unwrap();
                    *saved.lock().unwrap() = Some(std::mem::replace(world, World::empty()));
                    panic!("public remove original panic");
                })
            },
        );
        let panic =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.remove_entity(entity)))
                .unwrap_err();
        assert_eq!(
            panic.downcast_ref::<&'static str>(),
            Some(&"public remove original panic")
        );
        assert!(world.node_records().is_empty());
        let mut old = retained.lock().unwrap().take().unwrap();
        assert!(!Arc::ptr_eq(&original, &old.schedule().stage_plan()));
        assert!(old.contains_entity(entity));
        assert!(old.run_native_scene_system("tests.public-remove.panic-pending"));
        assert_eq!(systems.load(Ordering::SeqCst), 1);
    }
}
