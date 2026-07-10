use std::sync::{Arc, Mutex};
use std::thread;

use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsBodyType, PhysicsManager, PhysicsSettings, PhysicsSimulationMode,
    PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{NodeKind, RigidBodyComponent, RigidBodyType};
use zircon_runtime::scene::world::World;

use super::{apply_synchronized_bodies_to_scene, DefaultPhysicsManager, PhysicsBodyCommand};

#[test]
fn physics_manager_settings_and_clock_recover_poisoned_state_locks() {
    let manager = DefaultPhysicsManager::default();
    poison(manager.settings.clone());

    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 4,
            ..PhysicsSettings::default()
        })
        .unwrap();

    poison(manager.accumulators.clone());
    poison(manager.body_commands.clone());
    manager
        .queue_body_command(PhysicsBodyCommand::SetLinearVelocity {
            world: WorldHandle::new(7),
            entity: 1,
            velocity: [1.0, 0.0, 0.0],
        })
        .unwrap();
    let plan = manager.advance_clock(WorldHandle::new(7), 1.0 / 60.0);

    assert_eq!(PhysicsManager::settings(&manager).backend, "builtin");
    assert_eq!(plan.steps, 1);
    assert_eq!(manager.drain_body_commands(WorldHandle::new(7)).len(), 1);
}

#[test]
fn physics_manager_world_state_recovers_poisoned_state_locks() {
    let manager = DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let world = WorldHandle::new(11);

    poison(manager.synced_worlds.clone());
    poison(manager.contacts.clone());
    poison(manager.trigger_pairs.clone());
    poison(manager.triggers.clone());

    manager.sync_world(PhysicsWorldSyncState {
        world,
        ..PhysicsWorldSyncState::default()
    });

    assert_eq!(manager.synchronized_world(world).unwrap().world, world);
    assert!(manager.drain_contacts(world).is_empty());
    assert!(manager.drain_triggers(world).is_empty());
}

#[test]
fn physics_sync_to_scene_applies_synchronized_body_state() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .set_rigid_body(entity, Some(RigidBodyComponent::default()))
        .unwrap();
    let transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    let sync = PhysicsWorldSyncState {
        world: WorldHandle::new(19),
        bodies: vec![PhysicsBodySyncState {
            entity,
            body_type: PhysicsBodyType::Kinematic,
            transform,
            mass: 4.0,
            linear_velocity: [1.0, 2.0, 3.0],
            angular_velocity: [0.1, 0.2, 0.3],
            linear_damping: 0.25,
            angular_damping: 0.5,
            gravity_scale: 0.0,
            can_sleep: false,
            lock_translation: [true, false, true],
            lock_rotation: [false, true, false],
        }],
        ..PhysicsWorldSyncState::default()
    };

    apply_synchronized_bodies_to_scene(&mut world, &sync);

    assert_eq!(world.find_node(entity).unwrap().transform, transform);
    let body = world.rigid_body(entity).unwrap();
    assert_eq!(body.body_type, RigidBodyType::Kinematic);
    assert_eq!(body.mass, 4.0);
    assert_eq!(body.linear_velocity, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(body.angular_velocity, Vec3::new(0.1, 0.2, 0.3));
    assert!(!body.can_sleep);
    assert_eq!(body.lock_translation, [true, false, true]);
    assert_eq!(body.lock_rotation, [false, true, false]);
}

#[test]
fn unchanged_bodies_skip_sync() {
    let body = PhysicsBodySyncState {
        entity: 27,
        body_type: PhysicsBodyType::Dynamic,
        transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        mass: 2.0,
        linear_velocity: [4.0, 5.0, 6.0],
        angular_velocity: [0.1, 0.2, 0.3],
        linear_damping: 0.25,
        angular_damping: 0.5,
        gravity_scale: 1.0,
        can_sleep: true,
        lock_translation: [false; 3],
        lock_rotation: [false; 3],
    };

    let change = super::change_detection::detect_body_change(&body, &body);

    assert!(!change.requires_commands());
    assert!(!change.requires_recreation());
}

fn poison<T>(state: Arc<Mutex<T>>)
where
    T: Send + 'static,
{
    let result = thread::spawn(move || {
        let _guard = state.lock().unwrap();
        panic!("intentional poison for recovery coverage");
    })
    .join();

    assert!(result.is_err());
}
