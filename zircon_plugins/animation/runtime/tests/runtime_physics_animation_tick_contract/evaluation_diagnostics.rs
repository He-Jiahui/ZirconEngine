use super::*;

#[test]
fn invalid_clip_revision_emits_one_typed_diagnostic_per_revision() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = AssetUri::parse("res://animation/diagnostic.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/diagnostic.clip.zranim").unwrap();
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
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone())
            .with_source_hash("diagnostic-v1"),
        invalid_translation_clip(&skeleton_uri),
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
    let mut subscription = level.with_world_mut(|world| {
        let mut subscription = world.register_dormant_event_subscription::<
            zircon_plugin_animation_runtime::AnimationEvaluationDiagnostic,
        >();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    });

    runtime.tick_level_seconds(&level, 0.0).unwrap();
    let first = drain_evaluation_diagnostics(&level, &mut subscription);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].entity, entity);
    assert_eq!(first[0].clip.revision(), 1);
    assert!(matches!(
        first[0].error,
        zircon_plugin_animation_runtime::AnimationEvaluationError::InvalidChannelValueType {
            track_index: 0,
            channel: zircon_plugin_animation_runtime::AnimationTransformChannel::Translation,
            key_index: 0,
            role: zircon_plugin_animation_runtime::AnimationChannelDataRole::Value,
        }
    ));

    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert!(drain_evaluation_diagnostics(&level, &mut subscription).is_empty());

    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri)
            .with_source_hash("diagnostic-v2"),
        invalid_translation_clip(&skeleton_uri),
    );
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    let second = drain_evaluation_diagnostics(&level, &mut subscription);
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].clip.revision(), 2);
}

fn invalid_translation_clip(skeleton_uri: &AssetUri) -> zircon_runtime::asset::AnimationClipAsset {
    let mut clip = single_hand_translation_clip(skeleton_uri, 1.0);
    clip.tracks[0].translation.keys[0].value =
        zircon_runtime::asset::AnimationChannelValueAsset::Scalar(1.0);
    clip
}

fn drain_evaluation_diagnostics(
    level: &zircon_runtime::scene::LevelSystem,
    subscription: &mut zircon_runtime::scene::EventSubscription<
        zircon_plugin_animation_runtime::AnimationEvaluationDiagnostic,
    >,
) -> Vec<zircon_plugin_animation_runtime::AnimationEvaluationDiagnostic> {
    level.with_world_mut(|world| {
        world.update_events::<zircon_plugin_animation_runtime::AnimationEvaluationDiagnostic>();
        world
            .read_event_subscription(subscription)
            .cloned()
            .collect()
    })
}
