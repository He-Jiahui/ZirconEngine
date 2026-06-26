use super::*;

#[derive(Debug, PartialEq, Eq)]
struct DeferredMarker;

impl Component for DeferredMarker {}

fn resource_store_source() -> &'static str {
    concat!(
        include_str!("../../ecs/resource_store/mod.rs"),
        "\n",
        include_str!("../../ecs/resource_store/stored_resource.rs"),
        "\n",
        include_str!("../../ecs/resource_store/store.rs"),
    )
}

#[test]
fn resource_store_keeps_resources_by_concrete_type() {
    #[derive(Debug, PartialEq, Eq)]
    struct SceneFrameCounter(u32);

    let mut resources = ResourceStore::default();

    assert!(resources.is_empty());
    assert_eq!(resources.insert(SceneFrameCounter(1)), None);
    assert_eq!(
        resources.get::<SceneFrameCounter>(),
        Some(&SceneFrameCounter(1))
    );
    assert_eq!(
        resources.insert(SceneFrameCounter(2)),
        Some(SceneFrameCounter(1))
    );
    resources.get_mut::<SceneFrameCounter>().unwrap().0 += 1;
    assert_eq!(
        resources.remove::<SceneFrameCounter>(),
        Some(SceneFrameCounter(3))
    );
    assert!(!resources.contains::<SceneFrameCounter>());
}

#[test]
fn resource_store_type_names_preallocate_from_resource_count() {
    let resource_store_source = resource_store_source();

    assert!(
        resource_store_source.contains("let mut names = Vec::with_capacity(self.resources.len());")
    );
    assert!(resource_store_source.contains("for stored in self.resources.values()"));
    assert!(resource_store_source.contains("names.push(stored.type_name);"));
    assert!(!resource_store_source.contains("map(|stored| stored.type_name)"));
}

#[test]
fn typed_events_publish_on_update_and_keep_next_frame_events_separate() {
    let mut events = Events::<u32>::default();

    events.send(7);
    assert!(events.is_empty());

    events.update();
    assert_eq!(events.iter().copied().collect::<Vec<_>>(), vec![7]);

    events.send(9);
    assert_eq!(events.drain(), vec![7]);
    events.update();
    assert_eq!(events.drain(), vec![9]);
}

#[test]
fn apply_deferred_internal_system_flushes_queued_commands() {
    let mut world = crate::scene::World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    let mut system = SystemState::<CommandsParam>::new(&mut world).unwrap();

    system.run(&mut world, |mut commands| {
        commands.entity(entity).insert((DeferredMarker,));
    });

    assert!(world.get::<DeferredMarker>(entity).is_none());
    world.run_internal_scene_system(InternalSceneSystem::ApplyDeferred);

    assert_eq!(world.get::<DeferredMarker>(entity), Some(&DeferredMarker));
}

#[test]
fn event_store_tracks_each_event_type_independently() {
    #[derive(Debug, PartialEq, Eq)]
    struct Spawned(&'static str);
    #[derive(Debug, PartialEq, Eq)]
    struct Despawned(u64);

    let mut store = EventStore::default();
    store.register_reader::<Spawned>();
    store.register_reader::<Despawned>();
    store.send(Spawned("cube"));
    store.send(Despawned(42));

    assert!(store.events::<Spawned>().unwrap().is_empty());
    store.update::<Spawned>();
    assert_eq!(store.drain::<Spawned>(), vec![Spawned("cube")]);
    assert!(store.events::<Despawned>().unwrap().is_empty());

    store.update::<Despawned>();
    assert_eq!(store.drain::<Despawned>(), vec![Despawned(42)]);
    assert_eq!(store.registered_type_names().len(), 2);
}

#[test]
fn event_store_assigns_stable_dense_event_type_ids() {
    #[derive(Debug, PartialEq, Eq)]
    struct Spawned(&'static str);
    #[derive(Debug, PartialEq, Eq)]
    struct Despawned(u64);

    let mut store = EventStore::default();
    let spawned = store.register::<Spawned>();
    let despawned = store.register::<Despawned>();

    assert_eq!(spawned.raw(), 0);
    assert_eq!(despawned.raw(), 1);
    assert_eq!(store.register::<Spawned>(), spawned);
    assert_eq!(store.event_type_id::<Spawned>(), Some(spawned));
    assert_eq!(store.event_type_count(), 2);
}

#[test]
fn event_store_registered_channel_accepts_writes_before_reader_registered() {
    #[derive(Debug, PartialEq, Eq)]
    struct Spawned(&'static str);

    let mut store = EventStore::default();
    let spawned = store.register::<Spawned>();

    assert!(!store.is_active(spawned));
    assert_eq!(store.reader_count(spawned), Some(0));
    assert!(store.send_by_id(spawned, Spawned("before-reader")));
    store.update_by_id::<Spawned>(spawned);
    assert_eq!(store.drain::<Spawned>(), vec![Spawned("before-reader")]);

    assert_eq!(store.register_reader::<Spawned>(), spawned);
    assert!(store.is_active(spawned));
    assert_eq!(store.reader_count(spawned), Some(1));
    assert!(store.send_by_id(spawned, Spawned("after-reader")));
    store.update_by_id::<Spawned>(spawned);
    assert_eq!(store.drain::<Spawned>(), vec![Spawned("after-reader")]);
}

#[test]
fn dormant_subscription_connects_on_plugin_activate() {
    #[derive(Debug, PartialEq, Eq)]
    struct WeatherChanged(u32);

    let mut store = EventStore::default();
    let mut subscription = EventSubscription::<WeatherChanged>::new_dormant(&mut store);
    let event_type_id = subscription.event_type_id();

    assert_eq!(subscription.status(), EventSubscriptionStatus::Dormant);
    assert!(!store.is_active(event_type_id));
    assert_eq!(store.reader_count(event_type_id), Some(0));
    assert!(store.send_by_id(event_type_id, WeatherChanged(1)));
    store.update_by_id::<WeatherChanged>(event_type_id);
    assert!(subscription.read(&store).next().is_none());

    assert!(subscription.connect(&mut store));
    assert_eq!(subscription.status(), EventSubscriptionStatus::Connected);
    assert!(store.is_active(event_type_id));
    assert_eq!(store.reader_count(event_type_id), Some(1));
    assert!(subscription.read(&store).next().is_none());
    assert!(!subscription.connect(&mut store));
    assert_eq!(store.reader_count(event_type_id), Some(1));

    assert!(store.send_by_id(event_type_id, WeatherChanged(2)));
    store.update_by_id::<WeatherChanged>(event_type_id);
    let connected_read = subscription
        .read(&store)
        .map(|event| event.0)
        .collect::<Vec<_>>();
    assert_eq!(connected_read, vec![2]);

    assert!(subscription.disconnect(&mut store));
    assert_eq!(subscription.status(), EventSubscriptionStatus::Dormant);
    assert_eq!(store.reader_count(event_type_id), Some(0));
    assert!(!store.is_active(event_type_id));
    assert!(store.send_by_id(event_type_id, WeatherChanged(3)));
    store.update_by_id::<WeatherChanged>(event_type_id);

    assert!(subscription.connect(&mut store));
    assert!(subscription.read(&store).next().is_none());
    assert!(store.send_by_id(event_type_id, WeatherChanged(4)));
    store.update_by_id::<WeatherChanged>(event_type_id);
    let reconnected_read = subscription
        .read(&store)
        .map(|event| event.0)
        .collect::<Vec<_>>();
    assert_eq!(reconnected_read, vec![4]);
}

#[test]
fn event_channel_preallocates_next_queue_from_high_water() {
    let mut events = Events::<u32>::default();

    assert_eq!(events.send_batch(0_u32..64), 64);
    events.update();

    let metrics = events.capacity_metrics();
    assert_eq!(metrics.current_len, 64);
    assert_eq!(metrics.next_len, 0);
    assert!(metrics.high_water_len >= 64);
    assert!(
        metrics.next_capacity >= 64,
        "next frame write queue should be reserved from the observed high-water mark"
    );
}

#[test]
fn event_channel_shrinks_after_debounced_low_water_frames() {
    let mut events = Events::<u32>::default();
    events.send_batch(0_u32..64);
    events.update();
    let burst_capacity = events.capacity_metrics().retained_capacity();
    assert!(burst_capacity >= 64);

    for frame in 1..EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES {
        events.update();
        let metrics = events.capacity_metrics();
        assert_eq!(
            metrics.shrink_count, 0,
            "frame {frame} should stay inside the shrink debounce window"
        );
        assert_eq!(metrics.retained_capacity(), burst_capacity);
    }

    events.update();
    let metrics = events.capacity_metrics();
    assert_eq!(metrics.shrink_count, 1);
    assert_eq!(metrics.high_water_len, 0);
    assert_eq!(metrics.low_water_frames, 0);
    assert_eq!(metrics.retained_capacity(), 0);
}

#[test]
fn event_payload_profile_marks_large_payloads_for_arc_indirection() {
    struct InlinePayload([u8; EVENT_INLINE_PAYLOAD_MAX_BYTES]);
    struct LargeInlinePayload([u8; EVENT_INLINE_PAYLOAD_MAX_BYTES + 1]);
    struct LargePayloadByArc(Arc<[u8; 256]>);

    const _: [(); 1] =
        [(); (std::mem::size_of::<LargePayloadByArc>() <= EVENT_INLINE_PAYLOAD_MAX_BYTES) as usize];

    let inline = EventPayloadProfile::of::<InlinePayload>();
    assert_eq!(inline.size_bytes(), EVENT_INLINE_PAYLOAD_MAX_BYTES);
    assert_eq!(inline.storage(), EventPayloadStorage::Inline);
    assert!(!inline.requires_indirection());

    let large = EventPayloadProfile::of::<LargeInlinePayload>();
    assert_eq!(large.size_bytes(), EVENT_INLINE_PAYLOAD_MAX_BYTES + 1);
    assert_eq!(large.storage(), EventPayloadStorage::IndirectRecommended);
    assert!(large.requires_indirection());

    assert_eq!(
        EventPayloadProfile::of::<LargePayloadByArc>().storage(),
        EventPayloadStorage::Inline
    );

    let mut store = EventStore::default();
    let large_id = store.register::<LargeInlinePayload>();
    assert_eq!(store.payload_profile(large_id), Some(large));
}
