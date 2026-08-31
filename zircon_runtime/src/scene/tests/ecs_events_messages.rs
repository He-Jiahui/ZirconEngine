use crate::scene::World;
use crate::scene::ecs::{
    EventReaderParam, EventWriterParam, FunctionSceneSystem, Message, MessageReaderParam,
    MessageRetention, MessageWriterParam, ParamSet, ResParam, Resource, SceneSystemMetadata,
    ScheduleError, SystemOrderingConstraint, SystemRef, SystemStage, SystemState,
};

#[derive(Debug, PartialEq, Eq)]
struct FrameEvent(u32);

#[derive(Debug, PartialEq, Eq)]
struct IdleEvent;

#[derive(Debug, PartialEq, Eq)]
struct MissingRetirementResource;

impl Resource for MissingRetirementResource {}

#[derive(Debug, PartialEq, Eq)]
struct RetainedMessage(u32);

impl Message for RetainedMessage {}

#[derive(Debug, PartialEq, Eq)]
struct WeightedMessage {
    value: u32,
    retention_bytes: usize,
}

impl Message for WeightedMessage {
    fn retained_byte_size(&self) -> usize {
        self.retention_bytes
    }
}

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
fn system_state_retirement_releases_event_reader_lease_once() {
    let mut world = World::empty();
    let mut reader = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();
    let event_type_id = world.event_type_id::<FrameEvent>().unwrap();

    assert_eq!(world.event_reader_count(event_type_id), Some(1));
    reader.retire(&mut world);
    assert_eq!(world.event_reader_count(event_type_id), Some(0));

    reader.rebind(&mut world).unwrap();
    assert_eq!(world.event_reader_count(event_type_id), Some(1));

    reader.retire(&mut world);
    assert_eq!(world.event_reader_count(event_type_id), Some(0));
}

#[test]
fn dropped_system_state_releases_event_reader_lease() {
    let mut world = World::empty();
    let event_type_id;
    {
        let _reader = SystemState::<EventReaderParam<FrameEvent>>::new(&mut world).unwrap();
        event_type_id = world.event_type_id::<FrameEvent>().unwrap();
        assert_eq!(world.event_reader_count(event_type_id), Some(1));
    }

    assert_eq!(world.event_reader_count(event_type_id), Some(0));
}

#[test]
fn failed_tuple_system_param_initialization_releases_event_reader_lease() {
    let mut world = World::empty();

    let error = match SystemState::<(
        EventReaderParam<FrameEvent>,
        ResParam<MissingRetirementResource>,
    )>::new(&mut world)
    {
        Err(error) => error,
        Ok(_) => panic!("missing resource must reject system parameter initialization"),
    };
    assert!(
        error
            .to_string()
            .contains(std::any::type_name::<MissingRetirementResource>())
    );

    let event_type_id = world.event_type_id::<FrameEvent>().unwrap();
    assert_eq!(world.event_reader_count(event_type_id), Some(0));
}

#[test]
fn failed_param_set_initialization_releases_event_reader_lease() {
    let mut world = World::empty();

    let error = match SystemState::<
        ParamSet<(
            EventReaderParam<FrameEvent>,
            ResParam<MissingRetirementResource>,
        )>,
    >::new(&mut world)
    {
        Err(error) => error,
        Ok(_) => panic!("missing resource must reject parameter-set initialization"),
    };
    assert!(
        error
            .to_string()
            .contains(std::any::type_name::<MissingRetirementResource>())
    );

    let event_type_id = world.event_type_id::<FrameEvent>().unwrap();
    assert_eq!(world.event_reader_count(event_type_id), Some(0));
}

#[test]
fn failed_native_registration_retires_event_reader_lease() {
    let mut world = World::empty();
    let system = FunctionSceneSystem::<EventReaderParam<FrameEvent>, _>::new(
        SceneSystemMetadata::new("gameplay.event-reader-rollback", SystemStage::Update, 0)
            .with_constraint(SystemOrderingConstraint::After(SystemRef::System(
                "zircon.scene.events_update_all".to_string(),
            ))),
        &mut world,
        |_| {},
    )
    .unwrap();
    let event_type_id = world.event_type_id::<FrameEvent>().unwrap();

    assert!(matches!(
        world.register_boxed_native_system(Box::new(system)),
        Err(ScheduleError::CrossStageConstraint { .. })
    ));
    assert_eq!(world.event_reader_count(event_type_id), Some(0));
}

#[test]
fn unregister_native_system_retires_event_reader_lease() {
    let mut world = World::empty();
    world
        .register_native_system::<EventReaderParam<FrameEvent>, _>(
            "gameplay.event-reader-retirement",
            SystemStage::Update,
            0,
            |_| {},
        )
        .unwrap();
    let event_type_id = world.event_type_id::<FrameEvent>().unwrap();

    assert_eq!(world.event_reader_count(event_type_id), Some(1));
    assert!(
        world
            .unregister_native_system("gameplay.event-reader-retirement")
            .unwrap()
    );
    assert_eq!(world.event_reader_count(event_type_id), Some(0));
    assert!(
        !world
            .unregister_native_system("gameplay.event-reader-retirement")
            .unwrap()
    );
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
fn event_store_updates_only_dirty_channels_and_retires_delivered_generation() {
    let mut world = World::empty();
    world.register_event::<FrameEvent>();
    world.register_event::<IdleEvent>();

    world.update_all_events();
    assert_eq!(world.event_store().last_update_channel_visits(), 0);

    assert!(world.send_event(FrameEvent(11)));
    world.update_all_events();
    assert_eq!(world.event_store().last_update_channel_visits(), 1);
    assert_eq!(world.events::<FrameEvent>().unwrap().len(), 1);

    world.update_all_events();
    assert_eq!(world.event_store().last_update_channel_visits(), 1);
    assert!(world.events::<FrameEvent>().unwrap().is_empty());
}

#[test]
fn event_store_send_by_id_uses_registered_channel_guard_source() {
    let source = event_store_source();
    let send_by_id = event_store_section(source, "pub fn send_by_id", "pub fn send_batch");
    assert!(send_by_id.contains("if self.channel(event_type_id).is_none()"));
    assert!(send_by_id.contains("return false;"));
    assert!(send_by_id.contains("self.events_mut_by_id::<T>(event_type_id).send(event);"));
    assert!(!send_by_id.contains("is_active"));
    assert!(!send_by_id.contains("reader_count"));

    let send_batch_by_id =
        event_store_section(source, "pub fn send_batch_by_id", "pub fn update<T: Event>");
    assert!(send_batch_by_id.contains("if self.channel(event_type_id).is_none()"));
    assert!(send_batch_by_id.contains("return 0;"));
    assert!(send_batch_by_id.contains("if written > 0"));
    assert!(send_batch_by_id.contains("self.active_channels.insert(event_type_id);"));
    assert!(!send_batch_by_id.contains("is_active"));
    assert!(!send_batch_by_id.contains("reader_count"));
}

#[test]
fn event_store_update_all_uses_the_canonical_active_channel_worklist() {
    let source = event_store_source();
    let update_all = event_store_section(source, "pub fn update_all", "pub fn drain<T: Event>");
    assert!(update_all.contains("std::mem::take(&mut self.active_channels)"));
    assert!(update_all.contains("self.last_update_channel_visits = active_channels.len();"));
    assert!(update_all.contains("channel.events.requires_maintenance_erased()"));
    assert!(!update_all.contains("for channel in &mut self.channels"));
}

#[test]
fn first_stage_event_update_all_uses_builtin_source_path() {
    let registry_source = scene_system_registry_source();
    let builtin_systems = event_store_section(
        registry_source,
        "fn builtin_scene_systems",
        "fn validate_system_descriptor",
    );
    assert!(builtin_systems.contains("\"zircon.scene.events_update_all\""));
    assert!(builtin_systems.contains("SystemStage::First"));
    assert!(builtin_systems.contains("InternalSceneSystem::UpdateEvents"));
    assert!(builtin_systems.contains(".with_order(-10_000)"));

    let derived_state_source = world_derived_state_source();
    let run_internal = event_store_section(
        derived_state_source,
        "pub(crate) fn run_internal_scene_system",
        "pub(crate) fn run_internal_scene_systems_for_stage",
    );
    assert!(run_internal.contains("if system == InternalSceneSystem::UpdateEvents"));
    assert!(run_internal.contains("self.update_all_events();"));
    assert!(run_internal.contains("return;"));

    let world_events_source = world_events_source();
    let update_all_events = event_store_section(
        world_events_source,
        "pub fn update_all_events",
        "pub fn clear_events",
    );
    assert!(update_all_events.contains("self.events.update_all();"));
}

#[test]
fn event_subscription_source_keeps_dormant_reader_boundaries() {
    let source = event_subscription_source();
    let new_dormant = event_store_section(source, "pub fn new_dormant", "pub fn event_type_id");
    assert!(new_dormant.contains("event_type_id: store.register::<T>()"));
    assert!(new_dormant.contains("status: EventSubscriptionStatus::Dormant"));
    assert!(!new_dormant.contains("register_reader"));
    assert!(!new_dormant.contains("connect_reader"));

    let connect = event_store_section(source, "pub fn connect", "pub fn disconnect");
    assert!(connect.contains("let Some(reader_lease) = store.connect_reader(self.event_type_id)"));
    assert!(connect.contains("self.reader_lease = Some(reader_lease);"));
    assert!(connect.contains(".clear(store.events_by_id::<T>(self.event_type_id))"));
    assert!(connect.contains("self.status = EventSubscriptionStatus::Connected"));

    let disconnect = event_store_section(source, "pub fn disconnect", "pub fn read<'events>");
    assert!(disconnect.contains("let Some(mut reader_lease) = self.reader_lease.take()"));
    assert!(disconnect.contains("store.disconnect_reader(&mut reader_lease)"));
    assert!(disconnect.contains("self.reader_lease = Some(reader_lease);"));
    assert!(disconnect.contains("self.cursor.clear(None);"));
    assert!(disconnect.contains("self.status = EventSubscriptionStatus::Dormant"));

    let read = event_store_section(source, "pub fn read<'events>", "pub fn unread_count");
    assert!(read.contains("if !self.is_connected()"));
    assert!(read.contains("self.cursor.clear(None);"));
    assert!(read.contains("return EventReadIter::empty();"));
    assert!(read.contains(".read(store.events_by_id::<T>(self.event_type_id))"));
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
fn message_reader_does_not_acknowledge_unconsumed_iterator_tail() {
    let mut world = World::empty();
    let mut reader = SystemState::<MessageReaderParam<RetainedMessage>>::new(&mut world).unwrap();

    world.send_message(RetainedMessage(1));
    world.send_message(RetainedMessage(2));
    world.send_message(RetainedMessage(3));

    let first_page = reader.run(&mut world, |mut messages| {
        messages
            .read()
            .take(1)
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(first_page, vec![(0, 1)]);

    let remaining = reader.run(&mut world, |mut messages| {
        messages
            .read()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(remaining, vec![(1, 2), (2, 3)]);
}

#[test]
fn message_retention_budget_reports_slow_cursor_lag_without_replaying_evicted_entries() {
    let mut world = World::empty();
    world.configure_message_retention::<RetainedMessage>(MessageRetention::new(2, usize::MAX, 60));
    let mut reader = SystemState::<MessageReaderParam<RetainedMessage>>::new(&mut world).unwrap();

    world.send_message(RetainedMessage(1));
    world.send_message(RetainedMessage(2));
    world.send_message(RetainedMessage(3));

    let observed = reader.run(&mut world, |mut messages| {
        assert_eq!(messages.dropped_count(), 0);
        let observed = messages
            .read()
            .map(|(id, message)| (id.id(), message.0))
            .collect::<Vec<_>>();
        assert_eq!(messages.dropped_count(), 1);
        observed
    });
    assert_eq!(observed, vec![(1, 2), (2, 3)]);

    let metrics = world
        .message_retention_metrics::<RetainedMessage>()
        .unwrap();
    assert_eq!(metrics.retained_entries, 2);
    assert_eq!(
        metrics.retained_bytes,
        2 * std::mem::size_of::<RetainedMessage>()
    );
    assert_eq!(metrics.budget_dropped_entries, 1);
    assert_eq!(metrics.age_dropped_entries, 0);
}

#[test]
fn message_retention_byte_budget_uses_the_message_declared_charge() {
    let mut world = World::empty();
    world.configure_message_retention::<WeightedMessage>(MessageRetention::new(8, 10, 60));

    assert_eq!(
        world
            .send_message(WeightedMessage {
                value: 1,
                retention_bytes: 6,
            })
            .id(),
        0
    );
    assert_eq!(
        world
            .send_message(WeightedMessage {
                value: 2,
                retention_bytes: 6,
            })
            .id(),
        1
    );

    let retained = world
        .messages::<WeightedMessage>()
        .unwrap()
        .iter()
        .map(|(id, message)| (id.id(), message.value))
        .collect::<Vec<_>>();
    assert_eq!(retained, vec![(1, 2)]);

    let metrics = world
        .message_retention_metrics::<WeightedMessage>()
        .unwrap();
    assert_eq!(metrics.retained_entries, 1);
    assert_eq!(metrics.retained_bytes, 6);
    assert_eq!(metrics.budget_dropped_entries, 1);
    assert_eq!(metrics.budget_dropped_bytes, 6);
}

#[test]
fn first_stage_is_the_single_message_age_retirement_authority() {
    let mut world = World::empty();
    world.configure_message_retention::<RetainedMessage>(MessageRetention::new(8, usize::MAX, 0));
    let mut reader = SystemState::<MessageReaderParam<RetainedMessage>>::new(&mut world).unwrap();

    world.send_message(RetainedMessage(1));
    world.update_events::<FrameEvent>();
    assert_eq!(world.messages::<RetainedMessage>().unwrap().len(), 1);

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    let observed = reader.run(&mut world, |mut messages| {
        let observed = messages
            .read()
            .map(|(_id, message)| message.0)
            .collect::<Vec<_>>();
        assert_eq!(messages.dropped_count(), 1);
        observed
    });
    assert!(observed.is_empty());
    assert_eq!(
        world
            .message_retention_metrics::<RetainedMessage>()
            .unwrap()
            .age_dropped_entries,
        1
    );
}

#[test]
fn message_store_advances_only_active_retention_channels() {
    let mut world = World::empty();
    world.configure_message_retention::<RetainedMessage>(MessageRetention::new(8, usize::MAX, 60));

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 1);

    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 0);

    world.send_message(RetainedMessage(1));
    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 1);

    world.clear_messages::<RetainedMessage>();
    world.run_internal_scene_systems_for_stage(SystemStage::First);
    assert_eq!(world.last_message_advance_channel_visits(), 0);
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

#[test]
fn message_retention_source_has_single_first_stage_lifecycle_owner() {
    let queue_source = include_str!("../ecs/messages/queue.rs");
    assert!(queue_source.contains("VecDeque<MessageInstance<T>>"));
    assert!(queue_source.contains("enforce_budget"));
    assert!(queue_source.contains("retire_expired"));

    let message_id_source = include_str!("../ecs/messages/id.rs");
    assert!(message_id_source.contains("fn retained_byte_size(&self) -> usize"));

    let store_source = include_str!("../ecs/messages/store.rs");
    assert!(store_source.contains("pub fn advance_frame(&mut self)"));
    assert!(store_source.contains("advance_message_queue::<T>"));
    assert!(store_source.contains("std::mem::take(&mut self.active_channels)"));
    assert!(store_source.contains("last_advance_channel_visits"));

    let derived_state_source = world_derived_state_source();
    let run_internal = event_store_section(
        derived_state_source,
        "if system == InternalSceneSystem::UpdateEvents",
        "if !self.derived_state_dirty.should_run(system)",
    );
    assert!(run_internal.contains("self.advance_messages();"));
    assert!(run_internal.contains("self.update_all_events();"));
}

fn event_store_source() -> &'static str {
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

fn scene_system_registry_source() -> &'static str {
    include_str!("../ecs/scene_system_registry.rs")
}

fn world_derived_state_source() -> &'static str {
    include_str!("../world/derived_state.rs")
}

fn world_events_source() -> &'static str {
    include_str!("../world/events.rs")
}

fn event_subscription_source() -> &'static str {
    include_str!("../ecs/events/subscription.rs")
}

fn event_store_section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let after_start = source
        .split_once(start)
        .unwrap_or_else(|| panic!("event store source should contain {start}"))
        .1;
    after_start
        .split_once(end)
        .unwrap_or_else(|| {
            panic!("event store source section starting at {start} should contain {end}")
        })
        .0
}
