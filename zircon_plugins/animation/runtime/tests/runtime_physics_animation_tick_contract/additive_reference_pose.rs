use super::*;

#[test]
fn additive_clip_uses_bind_pose_as_reference_for_unkeyed_and_absolute_channels() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri =
        AssetUri::parse("res://animation/additive-reference.skeleton.zranim").unwrap();
    let base_uri = AssetUri::parse("res://animation/additive-reference-base.clip.zranim").unwrap();
    let additive_uri =
        AssetUri::parse("res://animation/additive-reference-layer.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/additive-reference.graph.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let graph_id = ResourceId::from_locator(&graph_uri);

    let mut skeleton = two_bone_skeleton();
    skeleton.bones[0].local_translation = [3.0, 0.0, 0.0];
    skeleton.bones[1].local_translation = [5.0, 0.0, 0.0];
    let resources = asset_manager.resource_manager();
    resources.register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        skeleton,
    );
    resources.register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&base_uri),
            ResourceKind::AnimationClip,
            base_uri.clone(),
        ),
        single_hand_translation_clip(&skeleton_uri, 20.0),
    );
    resources.register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&additive_uri),
            ResourceKind::AnimationClip,
            additive_uri.clone(),
        ),
        single_hand_translation_clip(&skeleton_uri, 7.0),
    );
    resources.register_ready(
        ResourceRecord::new(graph_id, ResourceKind::AnimationGraph, graph_uri),
        additive_mask_graph(&base_uri, &additive_uri),
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
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    runtime.tick_level_seconds(&level, 0.0).unwrap();

    let pose = level.animation_pose(entity).expect("additive pose");
    let root = pose.bones.iter().find(|bone| bone.name == "Root").unwrap();
    let hand = pose.bones.iter().find(|bone| bone.name == "Hand").unwrap();
    assert!(root
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(3.0, 0.0, 0.0), 1.0e-4));
    assert!(hand
        .local_transform
        .translation
        .abs_diff_eq(Vec3::new(22.0, 0.0, 0.0), 1.0e-4));
}
