use super::*;

#[test]
fn level_tick_invalidates_compiled_clip_after_remove_and_readd_with_reset_revision() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = AssetUri::parse("res://animation/cache-reset.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/cache-reset.clip.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let resources = asset_manager.resource_manager();
    resources.register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        single_hand_translation_clip(&skeleton_uri, 1.0),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Cube);
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    weight: 1.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert_hand_translation_x(&level, entity, 1.0);

    resources
        .remove_by_locator(&clip_uri)
        .expect("remove the first clip generation");
    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        single_hand_translation_clip(&skeleton_uri, 9.0),
    );
    assert_eq!(
        resources.registry().get(clip_id).unwrap().revision,
        1,
        "re-added resources currently begin a new revision sequence"
    );

    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert_hand_translation_x(&level, entity, 9.0);
}

#[test]
fn paused_clip_player_only_resamples_once_after_seek() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = AssetUri::parse("res://animation/paused-seek.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/paused-seek.clip.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let resources = asset_manager.resource_manager();
    resources.register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        single_hand_translation_clip(&skeleton_uri, 1.0),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Cube);
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    weight: 1.0,
                    looping: false,
                    playing: false,
                }),
            )
            .unwrap();
        entity
    });

    runtime.tick_level_seconds(&level, 0.0).unwrap();
    let initial_requests = clip_pose_request_count(&level);
    assert_eq!(initial_requests, 1, "initial paused pose is projected once");

    runtime.tick_level_seconds(&level, 1.0).unwrap();
    assert_eq!(
        clip_pose_request_count(&level),
        initial_requests,
        "unchanged paused player must not enqueue another pose sample"
    );
    assert_hand_translation_x(&level, entity, 1.0);

    let active_entity = level.with_world_mut(|world| {
        let active_entity = world.spawn_node(NodeKind::Cube);
        world
            .set_animation_skeleton(
                active_entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_player(
                active_entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.0,
                    weight: 1.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        active_entity
    });
    runtime.tick_level_seconds(&level, 1.0).unwrap();
    assert_hand_translation_x(&level, entity, 1.0);
    assert_eq!(
        clip_pose_request_count(&level),
        initial_requests + 1,
        "another entity's playback must not resample the paused player"
    );

    level.with_world_mut(|world| {
        world.set_animation_player(active_entity, None).unwrap();
    });

    level.with_world_mut(|world| {
        let mut player = world.animation_player(entity).cloned().unwrap();
        player.time_seconds = 0.5;
        world.set_animation_player(entity, Some(player)).unwrap();
    });
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert_eq!(
        clip_pose_request_count(&level),
        initial_requests + 2,
        "a paused seek must enqueue exactly one new pose sample"
    );

    runtime.tick_level_seconds(&level, 1.0).unwrap();
    assert_eq!(
        clip_pose_request_count(&level),
        initial_requests + 2,
        "the seek result must remain quiescent while paused"
    );

    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        single_hand_translation_clip(&skeleton_uri, 2.0),
    );
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert_hand_translation_x(&level, entity, 2.0);
    assert_eq!(
        clip_pose_request_count(&level),
        initial_requests + 3,
        "a paused resource revision must enqueue exactly one new pose sample"
    );

    runtime.tick_level_seconds(&level, 1.0).unwrap();
    assert_eq!(
        clip_pose_request_count(&level),
        initial_requests + 3,
        "the revised paused pose must remain quiescent after its single sample"
    );
}

#[test]
fn direct_clip_worker_keeps_one_hundred_and_one_thousand_instances_fair() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = AssetUri::parse("res://animation/worker-scale.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/worker-scale.clip.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let resources = asset_manager.resource_manager();
    resources.register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        single_hand_translation_clip(&skeleton_uri, 3.0),
    );

    let level = runtime.create_default_level().unwrap();
    let mut last_entity = None;
    let mut entities = Vec::new();
    let mut spawned = 0;
    for expected_instances in [1_usize, 100, 1_000] {
        level.with_world_mut(|world| {
            for _ in spawned..expected_instances {
                let next_entity = world.spawn_node(NodeKind::Cube);
                world
                    .set_animation_skeleton(
                        next_entity,
                        Some(AnimationSkeletonComponent {
                            skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                        }),
                    )
                    .unwrap();
                world
                    .set_animation_player(
                        next_entity,
                        Some(AnimationPlayerComponent {
                            clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                            playback_speed: 1.0,
                            time_seconds: 0.0,
                            weight: 1.0,
                            looping: true,
                            playing: true,
                        }),
                    )
                    .unwrap();
                entities.push(next_entity);
                last_entity = Some(next_entity);
            }
        });
        spawned = expected_instances;

        level.with_world_mut(|world| {
            for &entity in &entities {
                let mut player = world.animation_player(entity).cloned().unwrap();
                player.time_seconds = expected_instances as f32 * 0.25;
                world.set_animation_player(entity, Some(player)).unwrap();
            }
        });

        runtime.tick_level_seconds(&level, 0.0).unwrap();
        let worker_stats = level.with_world(|world| {
            world
                .resource::<zircon_plugin_animation_runtime::AnimationEvaluationPipeline>()
                .direct_clip_worker_stats()
        });
        assert_eq!(
            worker_stats.last_instance_count, expected_instances,
            "worker must process every direct-clip instance in the current frame"
        );
        assert!(worker_stats.last_shard_count > 0);
        assert_eq!(
            worker_stats.last_owner_submission_count, worker_stats.last_shard_count,
            "owner thread must submit exactly one bounded task per worker shard"
        );
        assert!(
            worker_stats.last_owner_submission_count
                <= zircon_plugin_animation_runtime::MAX_DIRECT_CLIP_WORKER_SHARDS,
            "owner-thread submission budget must remain independent of instance count"
        );
        assert!(
            worker_stats.last_max_shard_len - worker_stats.last_min_shard_len <= 1,
            "stable entity partitioning must keep shard work fair"
        );
    }

    assert_hand_translation_x(&level, last_entity.expect("scale fixture entity"), 3.0);
}

fn clip_pose_request_count(level: &zircon_runtime::scene::LevelSystem) -> u64 {
    level.with_world(|world| {
        world
            .resource::<zircon_plugin_animation_runtime::AnimationEvaluationPipeline>()
            .projection_stats()
            .clip_pose_request_count
    })
}

fn assert_hand_translation_x(
    level: &zircon_runtime::scene::LevelSystem,
    entity: u64,
    expected: f32,
) {
    let pose = level
        .animation_pose(entity)
        .expect("sampled animation pose");
    let hand = pose
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("hand pose row");
    assert!(
        (hand.local_transform.translation.x - expected).abs() <= 1.0e-4,
        "expected hand x={expected}, got {}",
        hand.local_transform.translation.x
    );
}
