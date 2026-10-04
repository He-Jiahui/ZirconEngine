#[cfg(feature = "animation")]
use crate::core::framework::animation::AnimationClipEventSamplingRange;
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
#[cfg(feature = "animation")]
use crate::core::resource::ResourceId;

use super::*;

fn poison_mutex<T>(mutex: &Mutex<T>) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = mutex.lock().unwrap();
        panic!("poison level system mutex");
    }));
    assert!(result.is_err());
}

#[test]
fn level_system_accessors_recover_poisoned_state_locks() {
    let level = LevelSystem::new(
        WorldHandle::new(42),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );

    poison_mutex(&level.inner);
    let entity = level.with_world_mut(|world| {
        world
            .spawn_node(crate::scene::NodeKind::Cube)
            .expect("test scene spawn should succeed")
    });
    assert!(level.snapshot().contains_entity(entity));

    #[cfg(feature = "animation")]
    {
        poison_mutex(&level.animation_state);
        assert_eq!(level.animation_playback_snapshot().generation(), 0);
        level.record_animation_requires_continuous_frame(true);
        assert!(level.animation_requires_continuous_frame());
        let replacement_epoch = level.capture_world_replacement_epoch();
        assert!(level.record_animation_pose_snapshot(replacement_epoch, Arc::default()));
    }

    poison_mutex(&level.frame_state);
    assert!(level.frame_state_snapshot().animation_poses().is_empty());

    poison_mutex(&level.physics_state);
    #[cfg(feature = "physics-contracts")]
    assert_eq!(level.last_physics_step_plan(), None);

    poison_mutex(&level.script_state);
    level.mark_script_binding_started(entity, "behavior");
    assert!(level.script_binding_started(entity, "behavior"));
    let script_generation = level.script_state_generation();

    poison_mutex(&level.metadata);
    level.set_metadata(LevelMetadata {
        display_name: Some("Recovered".to_string()),
        ..LevelMetadata::default()
    });
    assert_eq!(level.metadata().display_name.as_deref(), Some("Recovered"));

    poison_mutex(&level.lifecycle);
    level.set_lifecycle(LevelLifecycleState::Unloaded);
    assert_eq!(level.lifecycle(), LevelLifecycleState::Unloaded);

    poison_mutex(&level.subsystems);
    level.register_subsystem("physics");
    assert_eq!(
        level.registered_subsystems().as_ref(),
        ["physics".to_string()].as_slice()
    );

    level.replace_world_and_reset_runtime_state(World::empty());
    assert_eq!(level.script_state_generation(), script_generation + 1);
    assert!(!level.script_binding_started(entity, "behavior"));
}

#[test]
fn world_replacement_advances_generation_past_both_worlds() {
    let mut current = World::empty();
    current
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");
    current
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let current_generation = current.world_generation();
    let level = LevelSystem::new(
        WorldHandle::new(7),
        Arc::new(Mutex::new(current)),
        LevelMetadata::default(),
    );

    let mut replacement = World::empty();
    replacement
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let replacement_generation = replacement.world_generation();
    level.replace(replacement);

    assert_eq!(
        level.with_world(World::world_generation),
        current_generation.max(replacement_generation) + 1
    );
}

#[test]
fn world_replacement_stales_compiled_binding_when_entity_ids_are_reused() {
    let mut current = World::empty();
    let root = current
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = current
        .spawn_node(crate::scene::NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    current.rename_node(root, "Root").unwrap();
    current.rename_node(hero, "Hero").unwrap();
    current.set_parent_checked(hero, Some(root)).unwrap();
    let writer = current
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation").unwrap(),
        )
        .unwrap()
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(8),
        Arc::new(Mutex::new(current)),
        LevelMetadata::default(),
    );

    let mut replacement = World::empty();
    let replacement_root = replacement
        .spawn_node(crate::scene::NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let replacement_hero = replacement
        .spawn_node(crate::scene::NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    assert_eq!(root, replacement_root);
    assert_eq!(hero, replacement_hero);
    replacement.rename_node(replacement_root, "Root").unwrap();
    replacement.rename_node(replacement_hero, "Hero").unwrap();
    replacement
        .set_parent_checked(replacement_hero, Some(replacement_root))
        .unwrap();

    level.replace(replacement);

    assert!(level.with_world(|world| !writer.is_current_for(world)));
}

#[cfg(feature = "animation")]
#[test]
fn animation_clip_event_backlog_is_reset_with_the_replaced_world() {
    let level = LevelSystem::new(
        WorldHandle::new(9),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );
    let replacement_epoch = level.capture_world_replacement_epoch();
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            replacement_epoch,
            vec![vec![AnimationClipEventSamplingRange {
                entity: 17,
                clip_id: ResourceId::from_stable_label("animation.pending-event"),
                from_time_seconds: 0.0,
                to_time_seconds: 120.0,
                looping: true,
            }]],
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
            admitted_range_count: 1,
            deferred_range_count: 0,
            rejected_range_count: 0,
        },
    );

    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(1)
    );
    level.replace_world_and_reset_runtime_state(World::empty());
    let current_epoch = level.capture_world_replacement_epoch();
    assert_eq!(
        level.animation_clip_event_backlog_len(current_epoch),
        Some(0)
    );
    assert_eq!(level.animation_clip_event_drain_metrics().0, 0);
}

#[cfg(not(feature = "animation"))]
#[test]
fn level_system_constructs_and_replaces_world_without_animation() {
    let level = LevelSystem::new(
        WorldHandle::new(10),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );

    let before = level.world_generation();
    level.replace_world_and_reset_runtime_state(World::empty());

    assert!(level.world_generation() > before);
}
