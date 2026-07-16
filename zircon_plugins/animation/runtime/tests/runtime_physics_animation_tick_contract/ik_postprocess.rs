use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationClipAsset, AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationIkCommand, AnimationLookAtCommand, AnimationTargetId, AnimationTwoBoneIkCommand,
};
use zircon_runtime::core::manager::{animation_manager_handle, resolve_manager_service};
use zircon_runtime::core::math::{Mat4, Quat, Vec3};
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationSkeletonMarker, ResourceHandle, ResourceId, ResourceKind,
    ResourceRecord,
};
use zircon_runtime::scene::components::{
    AnimationPlayerComponent, AnimationSkeletonComponent, NodeKind,
};

use super::runtime_helpers::{runtime_asset_manager, runtime_with_physics_animation_scene_asset};

#[test]
fn queued_two_bone_ik_runs_after_pose_blend_before_pose_apply() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let assets = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/ik.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/ik.clip.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    assets.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        three_bone_chain(),
    );
    assets.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        AnimationClipAsset {
            name: Some("IkBindPose".to_string()),
            skeleton: AssetReference::from_locator(skeleton_uri),
            duration_seconds: 1.0,
            tracks: Vec::new(),
            event_tracks: Vec::new(),
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty);
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
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let animation = resolve_manager_service(
        &core,
        animation_manager_handle(&core).expect("animation manager handle"),
    )
    .unwrap();
    let target = Vec3::new(1.2, 0.8, 0.0);
    animation
        .queue_ik_command(AnimationIkCommand::TwoBone(AnimationTwoBoneIkCommand {
            world: level.world_handle(),
            entity,
            root: AnimationTargetId::from_segments(["Root"]),
            mid: AnimationTargetId::from_segments(["Root", "Mid"]),
            tip: AnimationTargetId::from_segments(["Root", "Mid", "Tip"]),
            target,
            pole: Some(Vec3::Z),
            weight: 1.0,
        }))
        .unwrap();

    runtime.tick_level_seconds(&level, 0.0).unwrap();

    let pose = level
        .animation_pose(entity)
        .expect("IK should retain the pose");
    let model = pose
        .bones
        .iter()
        .scan(Mat4::IDENTITY, |parent, bone| {
            *parent *= bone.local_transform.matrix();
            Some(*parent)
        })
        .collect::<Vec<_>>();
    let solved_tip = model[2].transform_point3(Vec3::ZERO);
    assert!(
        solved_tip.abs_diff_eq(target, 1.0e-4),
        "expected {target:?}, got {solved_tip:?}"
    );
    assert!(animation.drain_ik_commands(level.world_handle()).is_empty());
}

#[test]
fn queued_look_at_uses_model_space_target_and_clamps_final_pose() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let assets = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/look-at.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/look-at.clip.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    assets.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        AnimationSkeletonAsset {
            name: Some("LookAt".to_string()),
            bones: vec![bone("Root", None, [0.0, 0.0, 0.0])],
        },
    );
    assets.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        AnimationClipAsset {
            name: Some("LookAtBindPose".to_string()),
            skeleton: AssetReference::from_locator(skeleton_uri),
            duration_seconds: 1.0,
            tracks: Vec::new(),
            event_tracks: Vec::new(),
        },
    );
    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty);
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
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });
    let animation = resolve_manager_service(
        &core,
        animation_manager_handle(&core).expect("animation manager handle"),
    )
    .unwrap();
    animation
        .queue_ik_command(AnimationIkCommand::LookAt(AnimationLookAtCommand {
            world: level.world_handle(),
            entity,
            bone: AnimationTargetId::from_segments(["Root"]),
            target: Vec3::Y,
            axis: Vec3::X,
            clamp_degrees: 30.0,
            weight: 1.0,
        }))
        .unwrap();

    runtime.tick_level_seconds(&level, 0.0).unwrap();

    let pose = level
        .animation_pose(entity)
        .expect("look-at should retain pose");
    let angle = pose.bones[0]
        .local_transform
        .rotation
        .angle_between(Quat::IDENTITY)
        .to_degrees();
    assert!((angle - 30.0).abs() <= 1.0e-4, "got {angle}");
}

fn three_bone_chain() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("IkChain".to_string()),
        bones: vec![
            bone("Root", None, [0.0, 0.0, 0.0]),
            bone("Mid", Some(0), [1.0, 0.0, 0.0]),
            bone("Tip", Some(1), [1.0, 0.0, 0.0]),
        ],
    }
}

fn bone(
    name: &str,
    parent_index: Option<u32>,
    local_translation: [f32; 3],
) -> AnimationSkeletonBoneAsset {
    AnimationSkeletonBoneAsset {
        name: name.to_string(),
        parent_index,
        local_translation,
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0, 1.0, 1.0],
    }
}
