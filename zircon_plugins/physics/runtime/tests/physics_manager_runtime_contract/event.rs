use super::*;
use zircon_runtime::core::framework::physics::{
    PhysicsContactEvent, PhysicsTriggerEvent, PhysicsTriggerEventKind,
};

#[test]
fn trigger_lifecycle_enter_stay_exit_contract() {
    let runtime = create_runtime_with_scene_and_physics();
    configure_builtin_simulation(&runtime);
    let level = create_default_level(&runtime.handle()).unwrap();
    let (trigger, other) = spawn_trigger_pair(&level);

    tick_physics_level(&runtime, &level);
    assert_trigger(&level, PhysicsTriggerEventKind::Enter, trigger, other);

    tick_physics_level(&runtime, &level);
    assert_trigger(&level, PhysicsTriggerEventKind::Stay, trigger, other);

    level.with_world_mut(|world| {
        world
            .update_transform(other, Transform::from_translation(Vec3::new(8.0, 0.0, 0.0)))
            .unwrap();
    });
    tick_physics_level(&runtime, &level);
    assert_trigger(&level, PhysicsTriggerEventKind::Exit, trigger, other);
}

#[test]
fn physics_contact_and_trigger_events_reach_event_store() {
    let runtime = create_runtime_with_scene_and_physics();
    configure_builtin_simulation(&runtime);
    let level = create_default_level(&runtime.handle()).unwrap();
    let (trigger, other) = spawn_trigger_pair(&level);
    level.with_world_mut(|world| {
        let contact_peer = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                contact_peer,
                Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
            )
            .unwrap();
        world
            .set_collider(
                contact_peer,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 1.0 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
    });
    let (mut contacts, mut triggers) = level.with_world_mut(|world| {
        let mut contacts = world.register_dormant_event_subscription::<PhysicsContactEvent>();
        let mut triggers = world.register_dormant_event_subscription::<PhysicsTriggerEvent>();
        assert!(world.connect_event_subscription(&mut contacts));
        assert!(world.connect_event_subscription(&mut triggers));
        (contacts, triggers)
    });

    tick_physics_level(&runtime, &level);

    let (contact_events, trigger_events) = level.with_world_mut(|world| {
        world.update_events::<PhysicsContactEvent>();
        world.update_events::<PhysicsTriggerEvent>();
        let contacts = world
            .read_event_subscription(&mut contacts)
            .cloned()
            .collect::<Vec<_>>();
        let triggers = world
            .read_event_subscription(&mut triggers)
            .cloned()
            .collect::<Vec<_>>();
        (contacts, triggers)
    });
    assert!(!contact_events.is_empty());
    assert!(trigger_events.iter().any(|event| {
        event.kind == PhysicsTriggerEventKind::Enter
            && event.trigger_entity == trigger
            && event.other_entity == other
    }));
}

fn configure_builtin_simulation(runtime: &CoreRuntime) {
    runtime
        .resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();
}

fn spawn_trigger_pair(
    level: &LevelSystem,
) -> (
    zircon_runtime::scene::EntityId,
    zircon_runtime::scene::EntityId,
) {
    level.with_world_mut(|world| {
        let trigger = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                trigger,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 1.0 },
                    sensor: true,
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        let other = world.spawn_node(NodeKind::Cube);
        world
            .set_collider(
                other,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 1.0 },
                    ..ColliderComponent::default()
                }),
            )
            .unwrap();
        (trigger, other)
    })
}

fn assert_trigger(
    level: &LevelSystem,
    kind: PhysicsTriggerEventKind,
    trigger: zircon_runtime::scene::EntityId,
    other: zircon_runtime::scene::EntityId,
) {
    assert!(level.physics_triggers().iter().any(|event| {
        event.kind == kind && event.trigger_entity == trigger && event.other_entity == other
    }));
}
