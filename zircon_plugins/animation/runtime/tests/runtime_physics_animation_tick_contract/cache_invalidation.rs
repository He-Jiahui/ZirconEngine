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
