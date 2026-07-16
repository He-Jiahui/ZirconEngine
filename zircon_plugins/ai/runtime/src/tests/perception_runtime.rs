use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::core::framework::ai::{
    AiHearingStimulusEvent, AiHearingStimulusOrigin, AiManager, AiPerceptionSense,
};
use zircon_runtime::core::framework::animation::AnimationEventRecord;
use zircon_runtime::core::framework::physics::{
    PhysicsQueryInterface, PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsShapeCastHit,
    PhysicsShapeCastQuery, PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery,
    PHYSICS_QUERY_INTERFACE_ID,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::sound::SoundGameplayEmission;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::plugin::{
    BridgeImport, PluginModuleId, RuntimeExtensionRegistry, RuntimePluginRegistrationReport,
};
use zircon_runtime::scene::ecs::{SystemOrderingConstraint, SystemRef, SystemStage};
use zircon_runtime::scene::{
    create_default_level, module_descriptor as scene_module_descriptor, NodeKind, World,
    SCENE_MODULE_NAME,
};

use crate::perception::{
    ai_perception_component_descriptors, tick_perception, AiPerceptionChannels,
    AiPerceptionReceiver, AiPerceptionSource, AiTickBudget, HearingStimulusAdapter,
    PerceivedStimuli, AI_HEARING_ANIMATION_EVENT_NAME, AI_HEARING_PENDING_EVENT_CAPACITY,
    AI_HEARING_PENDING_EVENT_MAX_AGE_SECONDS, AI_PERCEPTION_RECEIVER_COMPONENT_TYPE,
    AI_PERCEPTION_SOURCE_COMPONENT_TYPE,
};
use crate::plugin::{collect_perception_hearing_events, PerceptionEventSubscriptions};
use crate::{
    plugin_registration, AiRuntimePlugin, AI_BEHAVIOR_TICK_SYSTEM, AI_PERCEPTION_TICK_SYSTEM,
};

const TEST_WORLD: WorldHandle = WorldHandle::new(41);

#[derive(Debug)]
struct PhysicsHits {
    hits: Vec<PhysicsRayCastHit>,
    calls: Arc<AtomicUsize>,
}

impl PhysicsQueryInterface for PhysicsHits {
    fn ray_cast(&self, _query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.hits.clone()
    }

    fn shape_overlap(&self, _query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit> {
        Vec::new()
    }

    fn shape_cast(&self, _query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit> {
        Vec::new()
    }
}

#[test]
fn scan_budget_caps_pairs_per_frame() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(2.0));
    for z in [-2.0, -3.0, -4.0, -5.0] {
        spawn_source(
            &mut world,
            Vec3::new(0.0, 0.0, z),
            AiPerceptionChannels::SIGHT,
        );
    }
    let mut budget = AiTickBudget::new(2);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    let first = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );

    assert_eq!(first.scanned_pairs, 2);
    assert_eq!(budget.consumed_pairs(), 2);
    assert_eq!(perceived.snapshot(receiver).unwrap().stimuli.len(), 2);
}

#[test]
fn scan_cursor_rotates_across_pair_budget() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(2.0));
    for z in [-2.0, -3.0, -4.0, -5.0] {
        spawn_source(
            &mut world,
            Vec3::new(0.0, 0.0, z),
            AiPerceptionChannels::SIGHT,
        );
    }
    let mut budget = AiTickBudget::new(2);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    for _ in 0..2 {
        let report = tick_perception(
            &world,
            TEST_WORLD,
            0.0,
            &mut budget,
            &mut perceived,
            &mut event_adapter,
            &[],
            None,
        );
        assert_eq!(report.scanned_pairs, 2);
    }

    assert_eq!(perceived.snapshot(receiver).unwrap().stimuli.len(), 4);
}

#[test]
fn stimulus_forgotten_after_timeout() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(0.25));
    let source = spawn_source(
        &mut world,
        Vec3::new(0.0, 0.0, -2.0),
        AiPerceptionChannels::SIGHT,
    );
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();
    tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );
    assert_eq!(perceived.snapshot(receiver).unwrap().stimuli.len(), 1);

    world.remove::<AiPerceptionSource>(source).unwrap();
    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.25,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );

    assert_eq!(report.forgotten_stimuli, 1);
    assert!(perceived.snapshot(receiver).unwrap().stimuli.is_empty());
}

#[test]
fn sight_uses_nearest_bridge_hit_even_when_results_are_unsorted() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    let source = spawn_source(
        &mut world,
        Vec3::new(0.0, 0.0, -3.0),
        AiPerceptionChannels::SIGHT,
    );
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();
    let calls = Arc::new(AtomicUsize::new(0));
    let (_registry, import, _provider_owner) = physics_import(Some(PhysicsHits {
        hits: vec![ray_hit(source, 3.0), ray_hit(999, 1.0)],
        calls: calls.clone(),
    }));

    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        Some(&import),
    );

    assert_eq!(report.physics_queries, 1);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(perceived.snapshot(receiver).unwrap().stimuli.is_empty());

    let (_registry, import, _provider_owner) = physics_import(Some(PhysicsHits {
        hits: vec![ray_hit(999, 4.0), ray_hit(source, 3.0)],
        calls,
    }));
    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        Some(&import),
    );
    assert_eq!(report.physics_queries, 1);
    assert!(perceived
        .snapshot(receiver)
        .unwrap()
        .stimuli
        .iter()
        .any(|stimulus| stimulus.source == source && stimulus.sense == AiPerceptionSense::Sight));
}

#[test]
fn no_physics_falls_back_to_cone_test() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    let source = spawn_source(
        &mut world,
        Vec3::new(0.0, 0.0, -3.0),
        AiPerceptionChannels::SIGHT,
    );
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();
    let (_registry, import, _provider_owner) = physics_import(None);

    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        Some(&import),
    );

    assert_eq!(report.fallback_sight_pairs, 1);
    let snapshot = perceived.snapshot(receiver).unwrap();
    assert!(snapshot.stimuli.iter().any(|stimulus| {
        stimulus.source == source && stimulus.sense == AiPerceptionSense::Sight
    }));
}

#[test]
fn sound_event_registers_hearing_stimulus() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    let source = world.spawn_node(NodeKind::Empty);
    let event = AiHearingStimulusEvent::sound_playback(source, Vec3::new(3.0, 0.0, 0.0), 0.75)
        .with_max_range(8.0);
    assert_eq!(event.origin, AiHearingStimulusOrigin::SoundPlayback);
    let mut budget = AiTickBudget::new(0);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[event],
        None,
    );

    assert_eq!(report.event_stimuli, 0);
    assert_eq!(event_adapter.pending_event_count(), 1);
    budget = AiTickBudget::new(1);
    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );
    assert_eq!(report.event_pairs, 1);
    assert_eq!(report.event_stimuli, 1);
    assert_eq!(event_adapter.pending_event_count(), 0);
    let snapshot = perceived.snapshot(receiver).unwrap();
    assert!(snapshot.stimuli.iter().any(|stimulus| {
        stimulus.source == source
            && stimulus.sense == AiPerceptionSense::Hearing
            && stimulus.strength == 0.75
    }));
}

#[test]
fn hearing_backlog_is_bounded_and_expires_stale_events() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    for _ in 0..31 {
        spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    }
    let source = world.spawn_node(NodeKind::Empty);
    let events = (0..=AI_HEARING_PENDING_EVENT_CAPACITY)
        .map(|_| AiHearingStimulusEvent::sound_playback(source, Vec3::ZERO, 1.0))
        .collect::<Vec<_>>();
    let mut budget = AiTickBudget::new(0);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &events,
        None,
    );

    assert_eq!(
        event_adapter.pending_event_count(),
        AI_HEARING_PENDING_EVENT_CAPACITY
    );
    assert_eq!(event_adapter.dropped_event_count(), 1);
    assert_eq!(event_adapter.pending_receiver_snapshot_count(), 1);

    budget = AiTickBudget::new(1);
    let report = tick_perception(
        &world,
        TEST_WORLD,
        AI_HEARING_PENDING_EVENT_MAX_AGE_SECONDS + 0.01,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );

    assert_eq!(report.event_pairs, 0);
    assert_eq!(event_adapter.pending_event_count(), 0);
    assert_eq!(
        event_adapter.expired_event_count(),
        AI_HEARING_PENDING_EVENT_CAPACITY as u64
    );
    assert!(perceived.snapshot(receiver).unwrap().stimuli.is_empty());
}

#[test]
fn hearing_backlog_uses_stable_receiver_ids_across_listener_churn() {
    let mut world = World::empty();
    let first = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(2.0));
    let second = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(2.0));
    let source = world.spawn_node(NodeKind::Empty);
    let event = AiHearingStimulusEvent::sound_playback(source, Vec3::ZERO, 1.0);
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    let first_report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[event],
        None,
    );
    assert_eq!(first_report.event_pairs, 1);
    assert!(perceived.snapshot(first).unwrap().stimuli.len() == 1);

    assert!(world.remove_entity(first));
    let late_receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(2.0));
    let second_report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );

    assert_eq!(second_report.event_pairs, 1);
    assert_eq!(event_adapter.pending_event_count(), 0);
    assert!(perceived.snapshot(second).unwrap().stimuli.len() == 1);
    assert!(perceived
        .snapshot(late_receiver)
        .unwrap()
        .stimuli
        .is_empty());
}

#[test]
fn sound_gameplay_emission_maps_to_hearing_with_preserved_age() {
    let source = 73;
    let emission = SoundGameplayEmission {
        sequence: 1,
        world: TEST_WORLD,
        source,
        position: [4.0, 2.0, 1.0],
        strength: 0.5,
        max_range: Some(12.0),
        emitted_at_seconds: 8.0,
    };

    let event = crate::perception::hearing_event_from_sound(&emission, 8.75)
        .expect("valid sound emission becomes hearing input");

    assert_eq!(event.source, source);
    assert_eq!(event.position, Vec3::new(4.0, 2.0, 1.0));
    assert_eq!(event.age_seconds, 0.75);
    assert_eq!(event.max_range, Some(12.0));
}

#[test]
fn event_and_static_pairs_share_budget_without_starvation() {
    let mut world = World::empty();
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    let sight_source = spawn_source(
        &mut world,
        Vec3::new(0.0, 0.0, -2.0),
        AiPerceptionChannels::SIGHT,
    );
    let hearing_source = world.spawn_node(NodeKind::Empty);
    let event =
        AiHearingStimulusEvent::sound_playback(hearing_source, Vec3::new(1.0, 0.0, 0.0), 1.0);
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    let first = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[event],
        None,
    );
    assert_eq!(first.scanned_pairs, 1);
    assert_eq!(first.event_pairs, 0);
    assert!(perceived
        .snapshot(receiver)
        .unwrap()
        .stimuli
        .iter()
        .any(|stimulus| stimulus.source == sight_source
            && stimulus.sense == AiPerceptionSense::Sight));

    let second = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );
    assert_eq!(second.scanned_pairs, 1);
    assert_eq!(second.event_pairs, 1);
    assert!(perceived
        .snapshot(receiver)
        .unwrap()
        .stimuli
        .iter()
        .any(|stimulus| {
            stimulus.source == hearing_source && stimulus.sense == AiPerceptionSense::Hearing
        }));
}

#[test]
fn authored_dynamic_components_feed_perception_scan() {
    let mut world = World::empty();
    for descriptor in ai_perception_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let receiver = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            receiver,
            AI_PERCEPTION_RECEIVER_COMPONENT_TYPE,
            toml::from_str::<toml::Value>(
                "sight_fov_degrees = 90.0\nsight_range = 20.0\nhearing_radius = 20.0\nforget_seconds = 1.0",
            )
            .unwrap()
            .try_into()
            .unwrap(),
        )
        .unwrap();
    let source = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(
            source,
            Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
        )
        .unwrap();
    world
        .set_dynamic_component(
            source,
            AI_PERCEPTION_SOURCE_COMPONENT_TYPE,
            toml::from_str::<toml::Value>(&format!(
                "channels = {}\nstrength = 0.8",
                AiPerceptionChannels::SIGHT.bits()
            ))
            .unwrap()
            .try_into()
            .unwrap(),
        )
        .unwrap();
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );

    assert_eq!(report.scanned_pairs, 1);
    assert!(perceived
        .snapshot(receiver)
        .unwrap()
        .stimuli
        .iter()
        .any(|stimulus| {
            stimulus.source == source
                && stimulus.sense == AiPerceptionSense::Sight
                && stimulus.strength == 0.8
        }));
}

#[test]
fn invalid_authored_dynamic_component_does_not_fall_back_to_typed_state() {
    let mut world = World::empty();
    for descriptor in ai_perception_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    let source = spawn_source(
        &mut world,
        Vec3::new(0.0, 0.0, -2.0),
        AiPerceptionChannels::SIGHT,
    );
    world
        .set_dynamic_component(
            source,
            AI_PERCEPTION_SOURCE_COMPONENT_TYPE,
            toml::from_str::<toml::Value>("channels = 'sight'\nstrength = 1.0")
                .unwrap()
                .try_into()
                .unwrap(),
        )
        .unwrap();
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    let report = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        None,
    );

    assert_eq!(report.scanned_pairs, 0);
    assert!(perceived.snapshot(receiver).unwrap().stimuli.is_empty());
}

#[test]
fn physics_import_tracks_disable_reload_and_revoke() {
    let mut world = World::empty();
    let _receiver = spawn_receiver(&mut world, Vec3::ZERO, receiver_config(1.0));
    let source = spawn_source(
        &mut world,
        Vec3::new(0.0, 0.0, -3.0),
        AiPerceptionChannels::SIGHT,
    );
    let blocked_calls = Arc::new(AtomicUsize::new(0));
    let (mut registry, import, provider_owner) = physics_import(Some(PhysicsHits {
        hits: vec![ray_hit(999, 1.0), ray_hit(source, 3.0)],
        calls: blocked_calls.clone(),
    }));
    let provider_owner = provider_owner.unwrap();
    let table = registry.frozen_bridge_table();
    let mut budget = AiTickBudget::new(1);
    let mut perceived = PerceivedStimuli::default();
    let mut event_adapter = HearingStimulusAdapter::default();

    let blocked = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        Some(&import),
    );
    assert_eq!(blocked.physics_queries, 1);
    assert_eq!(blocked_calls.load(Ordering::Relaxed), 1);

    table.set_owner_enabled(provider_owner, false);
    let disabled = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        Some(&import),
    );
    assert_eq!(disabled.fallback_sight_pairs, 1);
    assert_eq!(blocked_calls.load(Ordering::Relaxed), 1);

    let reloaded_calls = Arc::new(AtomicUsize::new(0));
    let slot = table.resolve_slot(PHYSICS_QUERY_INTERFACE_ID).unwrap();
    let provider: Arc<dyn PhysicsQueryInterface> = Arc::new(PhysicsHits {
        hits: Vec::new(),
        calls: reloaded_calls.clone(),
    });
    table
        .reload_provider::<dyn PhysicsQueryInterface>(slot, provider)
        .unwrap();
    table.set_owner_enabled(provider_owner, true);
    let reloaded = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        Some(&import),
    );
    assert_eq!(reloaded.physics_queries, 1);
    assert_eq!(reloaded.fallback_sight_pairs, 0);
    assert_eq!(reloaded_calls.load(Ordering::Relaxed), 1);

    registry.revoke_owner_registrations(provider_owner);
    let revoked = tick_perception(
        &world,
        TEST_WORLD,
        0.0,
        &mut budget,
        &mut perceived,
        &mut event_adapter,
        &[],
        Some(&import),
    );
    assert_eq!(revoked.fallback_sight_pairs, 1);
    assert_eq!(reloaded_calls.load(Ordering::Relaxed), 1);
}

#[test]
fn perception_system_registers_resources_contract_and_precedes_behavior() {
    let report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let system = report
        .extensions
        .plugin_runtime_systems()
        .find(|(_, system)| system.id == AI_PERCEPTION_TICK_SYSTEM)
        .map(|(_, system)| system)
        .expect("AI perception tick system");

    assert_eq!(system.stage, SystemStage::Update);
    assert!(system
        .constraints
        .contains(&SystemOrderingConstraint::Before(SystemRef::System(
            AI_BEHAVIOR_TICK_SYSTEM.to_string()
        ),)));
    assert!(report
        .package_manifest
        .dependencies
        .iter()
        .any(|dependency| {
            dependency.id == "physics"
                && !dependency.required
                && dependency.interfaces == ["physics.query.v1"]
        }));
    assert!(report
        .package_manifest
        .components
        .iter()
        .any(|component| { component.type_id == "ai.perception_source" }));
    assert!(report
        .package_manifest
        .components
        .iter()
        .any(|component| { component.type_id == "ai.perception_receiver" }));
    let resources = report
        .extensions
        .plugin_resources()
        .map(|(_, registration)| registration.type_name())
        .collect::<Vec<_>>();
    assert!(resources.contains(&std::any::type_name::<AiTickBudget>()));
    assert!(resources.contains(&std::any::type_name::<HearingStimulusAdapter>()));
    assert!(resources.contains(&std::any::type_name::<PerceivedStimuli>()));
}

#[test]
fn perception_scene_system_updates_the_shared_ai_manager() {
    let runtime = zircon_runtime::core::CoreRuntime::new();
    runtime.register_module(scene_module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let plugin = AiRuntimePlugin::new();
    let manager = plugin.manager();
    let mut registration = RuntimePluginRegistrationReport::from_plugin(&plugin);
    assert!(registration.is_success(), "{:?}", registration.diagnostics);

    let world_handle = level.world_handle();
    let receiver = level.with_world_mut(|world| {
        registration.extensions.apply_to_world(world).unwrap();
        let receiver = spawn_receiver(world, Vec3::ZERO, receiver_config(1.0));
        spawn_source(
            world,
            Vec3::new(0.0, 0.0, -2.0),
            AiPerceptionChannels::SIGHT,
        );
        receiver
    });

    let advance = runtime.advance_time_by(Duration::from_millis(16), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    let snapshot = manager
        .perception_snapshot(world_handle, receiver)
        .expect("perception system stores the receiver snapshot");
    assert_eq!(snapshot.stimuli.len(), 1);
    assert_eq!(snapshot.stimuli[0].sense, AiPerceptionSense::Sight);
}

#[test]
fn perception_subscription_skips_history_then_consumes_animation_bus_events() {
    let runtime = zircon_runtime::core::CoreRuntime::new();
    runtime.register_module(scene_module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let plugin = AiRuntimePlugin::new();
    let manager = plugin.manager();
    let mut registration = RuntimePluginRegistrationReport::from_plugin(&plugin);
    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    let world_handle = level.world_handle();
    let (receiver, source) = level.with_world_mut(|world| {
        registration.extensions.apply_to_world(world).unwrap();
        let receiver = spawn_receiver(world, Vec3::ZERO, receiver_config(1.0));
        let source = world.spawn_node(NodeKind::Empty);
        world
            .update_transform(
                source,
                Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            )
            .unwrap();
        world.send_event(AiHearingStimulusEvent::sound_playback(
            source,
            Vec3::new(2.0, 0.0, 0.0),
            0.25,
        ));
        (receiver, source)
    });

    let advance = runtime.advance_time_by(Duration::from_millis(16), 8);
    level.tick(&runtime.handle(), advance).unwrap();
    assert!(manager
        .perception_snapshot(world_handle, receiver)
        .unwrap()
        .stimuli
        .is_empty());

    level.with_world_mut(|world| {
        world.send_event(
            AnimationEventRecord::new(source, AI_HEARING_ANIMATION_EVENT_NAME).with_payload("0.75"),
        );
    });
    let advance = runtime.advance_time_by(Duration::from_millis(16), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    let snapshot = manager
        .perception_snapshot(world_handle, receiver)
        .expect("animation event creates a hearing snapshot");
    assert!(snapshot.stimuli.iter().any(|stimulus| {
        stimulus.source == source
            && stimulus.sense == AiPerceptionSense::Hearing
            && stimulus.strength == 0.75
    }));
}

#[test]
fn perception_subscription_reactivation_skips_disabled_history_without_reader_leak() {
    let mut world = World::empty();
    let mut subscriptions = PerceptionEventSubscriptions::default();
    subscriptions.begin_frame(&mut world, 1, 1);
    assert_eq!(subscriptions.sound_sequence(), None);
    subscriptions.advance_sound_sequence(7);
    let event_type = world.event_type_id::<AiHearingStimulusEvent>().unwrap();
    assert_eq!(world.event_reader_count(event_type), Some(0));

    world.send_event(AiHearingStimulusEvent::sound_playback(1, Vec3::ZERO, 1.0));
    world.update_events::<AiHearingStimulusEvent>();
    assert_eq!(subscriptions.read_hearing(&world).count(), 1);
    world.send_event(AiHearingStimulusEvent::sound_playback(2, Vec3::ZERO, 1.0));
    world.update_events::<AiHearingStimulusEvent>();
    assert!(!subscriptions.begin_frame(&mut world, 1, 2));
    assert_eq!(subscriptions.sound_sequence(), Some(7));

    subscriptions.begin_frame(&mut world, 2, 3);

    assert_eq!(subscriptions.sound_sequence(), None);
    subscriptions.advance_sound_sequence(11);
    assert_eq!(world.event_reader_count(event_type), Some(0));
    assert_eq!(subscriptions.read_hearing(&world).count(), 0);
    world.send_event(AiHearingStimulusEvent::sound_playback(3, Vec3::ZERO, 1.0));
    world.update_events::<AiHearingStimulusEvent>();
    assert_eq!(subscriptions.read_hearing(&world).count(), 1);

    world.send_event(AiHearingStimulusEvent::sound_playback(4, Vec3::ZERO, 1.0));
    world.update_events::<AiHearingStimulusEvent>();
    assert!(subscriptions.begin_frame(&mut world, 2, 5));
    assert_eq!(subscriptions.sound_sequence(), None);
    assert_eq!(subscriptions.read_hearing(&world).count(), 0);
    assert_eq!(world.event_reader_count(event_type), Some(0));
}

#[test]
fn sound_read_failure_does_not_consume_unrelated_hearing_bus_events() {
    let mut world = World::empty();
    let mut subscriptions = PerceptionEventSubscriptions::default();
    subscriptions.begin_frame(&mut world, 1, 1);
    world.send_event(AiHearingStimulusEvent::sound_playback(1, Vec3::ZERO, 1.0));
    world.update_events::<AiHearingStimulusEvent>();
    subscriptions.collect_bus_events(&world);

    let error = zircon_runtime::core::CoreError::Initialization(
        AI_PERCEPTION_TICK_SYSTEM.to_string(),
        "injected sound read failure".to_string(),
    );
    assert!(collect_perception_hearing_events(&mut subscriptions, Err(error), 0.0).is_err());

    world.update_events::<AiHearingStimulusEvent>();
    assert!(!subscriptions.begin_frame(&mut world, 1, 2));
    subscriptions.advance_pending_bus_time(0.25);
    subscriptions.collect_bus_events(&world);
    let (events, sound_read, dropped_bus_events) =
        collect_perception_hearing_events(&mut subscriptions, Ok(None), 0.25).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, 1);
    assert_eq!(events[0].age_seconds, 0.25);
    assert!(sound_read.is_none());
    assert_eq!(dropped_bus_events, 0);
}

#[test]
fn perception_subscription_bus_backlog_is_bounded_and_reset_clears_diagnostics() {
    let mut world = World::empty();
    let mut subscriptions = PerceptionEventSubscriptions::default();
    subscriptions.begin_frame(&mut world, 1, 1);
    for source in 0..(AI_HEARING_PENDING_EVENT_CAPACITY as u64 + 2) {
        world.send_event(AiHearingStimulusEvent::sound_playback(
            source,
            Vec3::ZERO,
            1.0,
        ));
    }
    world.update_events::<AiHearingStimulusEvent>();

    subscriptions.collect_bus_events(&world);

    assert_eq!(
        subscriptions.pending_bus_event_count(),
        AI_HEARING_PENDING_EVENT_CAPACITY
    );
    assert_eq!(subscriptions.dropped_bus_event_count(), 2);
    assert!(subscriptions.begin_frame(&mut world, 2, 2));
    assert_eq!(subscriptions.pending_bus_event_count(), 0);
    assert_eq!(subscriptions.dropped_bus_event_count(), 0);
}

fn physics_import(
    provider: Option<PhysicsHits>,
) -> (
    RuntimeExtensionRegistry,
    BridgeImport<dyn PhysicsQueryInterface>,
    Option<PluginModuleId>,
) {
    let mut registry = RuntimeExtensionRegistry::default();
    let consumer = registry
        .intern_plugin_module("test.ai.perception.consumer")
        .unwrap();
    let import = registry
        .import_interface::<dyn PhysicsQueryInterface>(consumer)
        .unwrap();
    let provider_owner = provider.map(|provider| {
        let owner = registry
            .intern_plugin_module("test.physics.query.provider")
            .unwrap();
        let provider: Arc<dyn PhysicsQueryInterface> = Arc::new(provider);
        registry
            .export_interface::<dyn PhysicsQueryInterface>(owner, provider)
            .unwrap();
        owner
    });
    registry.finalize();
    (registry, import, provider_owner)
}

fn ray_hit(entity: u64, distance: f32) -> PhysicsRayCastHit {
    PhysicsRayCastHit {
        entity,
        distance,
        position: [0.0, 0.0, -distance],
        normal: [0.0, 0.0, 1.0],
    }
}

fn receiver_config(forget_seconds: f32) -> AiPerceptionReceiver {
    AiPerceptionReceiver {
        sight_fov_degrees: 90.0,
        sight_range: 20.0,
        hearing_radius: 20.0,
        forget_seconds,
    }
}

fn spawn_receiver(world: &mut World, position: Vec3, receiver: AiPerceptionReceiver) -> u64 {
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(entity, Transform::from_translation(position))
        .unwrap();
    world.insert(entity, receiver).unwrap();
    entity
}

fn spawn_source(world: &mut World, position: Vec3, channels: AiPerceptionChannels) -> u64 {
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(entity, Transform::from_translation(position))
        .unwrap();
    world
        .insert(
            entity,
            AiPerceptionSource {
                channels,
                strength: 1.0,
            },
        )
        .unwrap();
    entity
}
