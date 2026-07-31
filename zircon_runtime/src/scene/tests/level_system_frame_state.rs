#[cfg(feature = "animation")]
use std::collections::BTreeMap;
#[cfg(feature = "animation")]
use std::sync::Arc;

#[cfg(feature = "animation")]
use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
#[cfg(feature = "animation")]
use crate::core::math::{Transform, Vec3};
use crate::scene::DefaultLevelManager;
#[cfg(feature = "animation")]
use crate::scene::World;

#[cfg(feature = "animation")]
fn pose_output() -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: Some("Locomotion".to_string()),
        bones: vec![AnimationPoseBone {
            name: "Root".to_string(),
            local_transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        }],
    }
}

#[test]
#[cfg(feature = "animation")]
fn level_frame_snapshot_reuses_sealed_pose_handle_until_world_replacement() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial = level.frame_state_snapshot();
    let world_generation = level.world_generation();
    let poses = Arc::new(BTreeMap::from([(17, pose_output())]));

    assert!(level.record_animation_pose_snapshot(world_generation, Arc::clone(&poses)));
    let published = level.frame_state_snapshot();
    assert_eq!(
        published.animation_generation(),
        initial.animation_generation() + 1
    );
    assert!(Arc::ptr_eq(published.animation_poses(), &poses));

    assert!(level.record_animation_pose_snapshot(world_generation, Arc::clone(&poses)));
    let stable = level.frame_state_snapshot();
    assert!(Arc::ptr_eq(&published, &stable));
    assert!(Arc::ptr_eq(stable.animation_poses(), &poses));

    level.replace_world_and_reset_runtime_state(World::empty());
    let reset = level.frame_state_snapshot();
    assert!(!Arc::ptr_eq(&published, &reset));
    assert_eq!(
        reset.world_generation(),
        level.with_world(World::world_generation)
    );
    assert!(reset.animation_poses().is_empty());
}

#[test]
#[cfg(feature = "animation")]
fn level_frame_snapshot_rejects_a_pose_payload_from_a_retired_world_generation() {
    let level = DefaultLevelManager::default().create_default_level();
    let retired_generation = level.world_generation();
    let stale_poses = Arc::new(BTreeMap::from([(17, pose_output())]));

    level.replace_world_and_reset_runtime_state(World::empty());
    assert!(!level.record_animation_pose_snapshot(retired_generation, stale_poses));
    assert!(level.frame_state_snapshot().animation_poses().is_empty());
}

#[test]
#[cfg(feature = "animation")]
fn level_replace_retires_the_sealed_pose_payload_through_the_legacy_entry_point() {
    let level = DefaultLevelManager::default().create_default_level();
    let world_generation = level.world_generation();
    let poses = Arc::new(BTreeMap::from([(17, pose_output())]));

    assert!(level.record_animation_pose_snapshot(world_generation, poses));
    let published = level.frame_state_snapshot();
    assert!(!published.animation_poses().is_empty());

    level.replace(World::empty());
    let replaced = level.frame_state_snapshot();
    assert!(!Arc::ptr_eq(&published, &replaced));
    assert!(replaced.animation_poses().is_empty());
    assert!(replaced.world_generation() > world_generation);
}

#[test]
fn level_script_binding_query_uses_the_borrowed_key_without_call_site_allocation() {
    let level = DefaultLevelManager::default().create_default_level();
    let entity = 17;

    level.mark_script_binding_started(entity, "player-controller");
    assert!(level.script_binding_started(entity, "player-controller"));
    assert!(!level.script_binding_started(entity, "camera-controller"));

    let source = include_str!("../level_system/frame_state.rs");
    assert!(source.contains("bindings.contains(binding_key)"));
    assert!(!source.contains("binding_key.to_string()"));
}

#[test]
fn level_render_extract_projects_sealed_pose_payload_after_the_world_lane() {
    let source = include_str!("../level_system_render_extract.rs");

    assert!(source.contains("let candidate_entities = frame_state"));
    assert!(source.contains("let (mut extract, skeletons) = self.with_world_mut"));
    assert!(source.contains("skeletons.into_iter()"));
    assert!(source.contains("pose: pose.clone()"));
}

#[test]
#[cfg(feature = "animation")]
fn level_world_replacement_and_pose_publication_share_a_generation_commit_order() {
    let source = include_str!("../level_system.rs");
    let replacement = source
        .split("pub fn replace_world_and_reset_runtime_state")
        .nth(1)
        .and_then(|section| section.split("pub fn with_world").next())
        .expect("read world replacement implementation");
    let publication = source
        .split("pub fn record_animation_pose_snapshot")
        .nth(1)
        .and_then(|section| {
            section
                .split("pub fn record_animation_playback_times")
                .next()
        })
        .expect("read animation publication implementation");

    let replacement_world_lock = replacement
        .find("let mut current = self.lock_world();")
        .expect("replacement holds the World lane");
    let replacement_frame_lock = replacement
        .find("let mut frame_state = self.lock_frame_state();")
        .expect("replacement publishes the retirement frame while holding the World lane");
    assert!(replacement_world_lock < replacement_frame_lock);

    let publication_world_lock = publication
        .find("let world = self.lock_world();")
        .expect("publication validates against the World lane");
    let publication_frame_lock = publication
        .find("let mut current = self.lock_frame_state();")
        .expect("publication commits while the World lane remains held");
    assert!(publication_world_lock < publication_frame_lock);
    assert!(publication.contains("world_generation: u64"));
    assert!(publication.contains("if world_generation != world.world_generation()"));
    assert!(!publication.contains("self.with_world(World::world_generation)"));
}
