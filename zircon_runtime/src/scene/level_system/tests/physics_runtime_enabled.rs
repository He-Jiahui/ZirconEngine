use std::sync::{Arc, Mutex};

use crate::core::framework::physics::{
    PhysicsContactEvent, PhysicsTriggerEvent, PhysicsWorldStepPlan,
};
use crate::core::framework::scene::WorldHandle;
use crate::scene::world::World;

use super::LevelSystem;
use crate::scene::level_system::LevelMetadata;

#[test]
fn physics_runtime_state_records_and_resets_with_the_level() {
    let level = LevelSystem::new(
        WorldHandle::new(42),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );
    let plan = PhysicsWorldStepPlan {
        steps: 1,
        step_seconds: 1.0 / 60.0,
        remaining_seconds: 0.0,
        interpolation_alpha: 0.0,
    };

    assert!(level.record_physics_step_if_replacement_epoch(
        level.capture_world_replacement_epoch(),
        plan,
        Vec::<PhysicsContactEvent>::new(),
        Vec::<PhysicsTriggerEvent>::new(),
    ));

    assert_eq!(level.last_physics_step_plan(), Some(plan));
    assert!(level.physics_contacts().is_empty());
    assert!(level.physics_triggers().is_empty());

    level.replace_world_and_reset_runtime_state(World::empty());
    assert_eq!(level.last_physics_step_plan(), None);
}

#[test]
fn physics_runtime_state_seals_event_payloads_and_reuses_stable_handles() {
    let level = LevelSystem::new(
        WorldHandle::new(43),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );
    let plan = PhysicsWorldStepPlan {
        steps: 1,
        step_seconds: 1.0 / 60.0,
        remaining_seconds: 0.0,
        interpolation_alpha: 0.0,
    };
    let initial = level.physics_frame_snapshot();

    let replacement_epoch = level.capture_world_replacement_epoch();
    assert!(level.record_physics_step_if_replacement_epoch(
        replacement_epoch,
        plan,
        Vec::<PhysicsContactEvent>::new(),
        Vec::<PhysicsTriggerEvent>::new(),
    ));
    let published = level.physics_frame_snapshot();
    assert_eq!(initial.generation(), 0);
    assert_eq!(published.generation(), 1);
    assert_eq!(published.step_plan(), Some(plan));
    assert!(published.contacts().is_empty());
    assert!(published.triggers().is_empty());

    assert!(level.record_physics_step_if_replacement_epoch(
        replacement_epoch,
        plan,
        Vec::<PhysicsContactEvent>::new(),
        Vec::<PhysicsTriggerEvent>::new(),
    ));
    assert!(Arc::ptr_eq(&published, &level.physics_frame_snapshot()));
    assert!(Arc::ptr_eq(published.contacts(), &level.physics_contacts()));
    assert!(Arc::ptr_eq(published.triggers(), &level.physics_triggers()));

    level.replace_world_and_reset_runtime_state(World::empty());
    let reset = level.physics_frame_snapshot();
    assert_eq!(reset.generation(), published.generation() + 1);
    assert!(!Arc::ptr_eq(&published, &reset));
    assert!(reset.contacts().is_empty());
    assert_eq!(published.step_plan(), Some(plan));
}

#[test]
fn physics_runtime_state_rejects_publication_from_a_retired_replacement_epoch() {
    let level = LevelSystem::new(
        WorldHandle::new(44),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );
    let retired_epoch = level.capture_world_replacement_epoch();
    level.replace_world_and_reset_runtime_state(World::empty());

    assert!(!level.record_physics_step_if_replacement_epoch(
        retired_epoch,
        PhysicsWorldStepPlan::default(),
        Vec::new(),
        Vec::new(),
    ));
    assert_eq!(level.last_physics_step_plan(), None);
}
