use std::sync::TryLockError;

use super::{WorldPublicationError, WorldPublicationSource};
use crate::scene::components::Name;
use crate::scene::{
    DefaultLevelManager, EntityId, LevelLifecycleState, LevelMetadata, LevelSystem, NodeKind, World,
};

fn publication_fixture() -> (DefaultLevelManager, LevelSystem, EntityId) {
    let manager = DefaultLevelManager::default();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let level = manager.create_level(world, LevelMetadata::default());
    (manager, level, entity)
}

fn assert_rejected_without_callback(
    source: &WorldPublicationSource,
    expected: WorldPublicationError,
) {
    let mut callbacks = 0;
    let result = source.publish(|_| callbacks += 1);
    assert_eq!(result, Err(expected));
    assert_eq!(callbacks, 0, "rejected preparation must not publish");
}

#[test]
fn unchanged_capture_publishes_from_the_live_world_under_both_owner_locks() {
    let (_manager, level, entity) = publication_fixture();
    let source = level.capture();
    let cloned_source = source.clone();
    assert_eq!(source.generation(), level.world_generation());
    assert_eq!(source.snapshot().world_generation(), source.generation());
    assert_eq!(
        source.replacement_epoch(),
        level.capture_world_replacement_epoch()
    );
    let expected_name = source.snapshot().get::<Name>(entity).unwrap().0.clone();
    // Repeating the same lifecycle state must not revoke the original capture.
    level.set_lifecycle(LevelLifecycleState::Loaded);
    let owner = source.owner.upgrade().unwrap();
    let lifecycle = source.lifecycle.upgrade().unwrap();
    let mut callbacks = 0;
    let name = source
        .publish(|live| {
            callbacks += 1;
            assert!(!std::ptr::eq(live, source.snapshot()));
            assert_eq!(live.world_generation(), source.generation());
            assert!(matches!(
                owner.world.try_lock(),
                Err(TryLockError::WouldBlock)
            ));
            assert!(matches!(
                lifecycle.try_lock(),
                Err(TryLockError::WouldBlock)
            ));
            live.get::<Name>(entity).unwrap().0.clone()
        })
        .unwrap();
    assert_eq!(name, expected_name);
    assert_eq!(callbacks, 1);
    assert_eq!(
        cloned_source.publish(World::world_generation),
        Ok(source.generation())
    );
}

#[test]
fn world_mutation_rejects_the_old_generation_without_running_publication() {
    let (_manager, level, original) = publication_fixture();
    let source = level.capture();
    let captured_generation = source.generation();
    let captured_replacement = source.replacement_epoch();
    let added = level.with_world_mut(|world| world.spawn_node(NodeKind::Empty).unwrap());
    let actual_generation = level.world_generation();
    assert!(actual_generation > captured_generation);
    assert_eq!(
        level.capture_world_replacement_epoch(),
        captured_replacement
    );
    assert!(source.snapshot().get::<Name>(original).is_some());
    assert!(source.snapshot().get::<Name>(added).is_none());
    assert_eq!(source.snapshot().world_generation(), captured_generation);
    assert_rejected_without_callback(
        &source,
        WorldPublicationError::WorldGenerationChanged {
            expected: captured_generation,
            actual: actual_generation,
        },
    );

    let fresh = level.capture();
    assert_eq!(
        fresh.publish(|live| live.get::<Name>(added).is_some()),
        Ok(true)
    );
}

#[test]
fn world_replacement_revokes_the_original_identity_and_accepts_a_fresh_capture() {
    let (_manager, level, original) = publication_fixture();
    let source = level.capture();
    let captured_replacement = source.replacement_epoch();
    level.replace_world_and_reset_runtime_state(World::empty());
    let actual_replacement = level.capture_world_replacement_epoch();
    assert_eq!(
        actual_replacement,
        captured_replacement.checked_add(1).unwrap()
    );
    assert!(level.world_generation() > source.generation());
    assert!(source.snapshot().get::<Name>(original).is_some());
    assert!(level.with_world(|world| world.get::<Name>(original).is_none()));
    assert_rejected_without_callback(
        &source,
        WorldPublicationError::WorldReplacementChanged {
            expected: captured_replacement,
            actual: actual_replacement,
        },
    );

    let fresh = level.capture();
    assert_eq!(fresh.replacement_epoch(), actual_replacement);
    assert_eq!(
        fresh.publish(World::world_generation),
        Ok(fresh.generation())
    );
}

#[test]
fn unload_reload_aba_revokes_work_even_when_world_and_replacement_are_unchanged() {
    let (_manager, level, _entity) = publication_fixture();
    let shared_level = level.clone();
    let source = level.capture();
    let captured_generation = source.generation();
    let captured_replacement = source.replacement_epoch();
    shared_level.set_lifecycle(LevelLifecycleState::Unloaded);
    assert_eq!(level.lifecycle(), LevelLifecycleState::Unloaded);
    assert_rejected_without_callback(&source, WorldPublicationError::LevelUnloaded);

    shared_level.set_lifecycle(LevelLifecycleState::Loaded);
    assert_eq!(level.lifecycle(), LevelLifecycleState::Loaded);
    assert_eq!(level.world_generation(), captured_generation);
    assert_eq!(
        level.capture_world_replacement_epoch(),
        captured_replacement
    );
    let fresh = level.capture();
    assert!(fresh.lifecycle_epoch > source.lifecycle_epoch);
    assert_rejected_without_callback(
        &source,
        WorldPublicationError::LifecycleChanged {
            expected: source.lifecycle_epoch,
            actual: fresh.lifecycle_epoch,
        },
    );
    assert_eq!(
        fresh.publish(World::world_generation),
        Ok(captured_generation)
    );
}

#[test]
fn captures_do_not_keep_the_original_level_owner_alive() {
    let (manager, level, entity) = publication_fixture();
    let remaining_level = level.clone();
    let source = level.capture();
    let cloned_source = source.clone();
    drop(level);
    drop(manager);
    // A real LevelSystem clone, rather than the registry or snapshot, owns this lifetime.
    assert_eq!(
        source.publish(World::world_generation),
        Ok(source.generation())
    );
    drop(remaining_level);
    assert!(source.owner.upgrade().is_none());
    assert!(source.lifecycle.upgrade().is_none());
    assert_rejected_without_callback(&source, WorldPublicationError::WorldOwnerDropped);
    assert_rejected_without_callback(&cloned_source, WorldPublicationError::WorldOwnerDropped);
    assert!(source.snapshot().get::<Name>(entity).is_some());
    assert!(cloned_source.snapshot().get::<Name>(entity).is_some());
}
