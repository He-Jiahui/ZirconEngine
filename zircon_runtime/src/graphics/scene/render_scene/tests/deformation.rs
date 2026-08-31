use std::sync::Arc;

use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use crate::core::framework::render::RenderMeshBounds;
use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::ResourceId;

use super::super::{
    RenderSceneDelta, RenderScenePrimitive, RenderScenePrimitiveDirtyFlags,
    RenderScenePrimitiveField, RenderScenePrimitiveLocalBounds, RenderSceneSkeletalPose,
};
use super::fixtures::{
    stable_key, test_descriptor, test_primitive, test_primitive_with, test_revisions, test_scene,
};

#[test]
fn render_scene_skeletal_pose_retains_the_sealed_pose_handle() {
    let pose = Arc::new(AnimationPoseOutput {
        source: AnimationPoseSource::Graph,
        active_state: Some("locomotion".to_owned()),
        bones: Vec::new(),
    });

    let scene_pose = RenderSceneSkeletalPose::new(
        ResourceId::from_stable_label("tests/render-scene/sealed-skeleton"),
        Arc::clone(&pose),
    );

    assert!(Arc::ptr_eq(scene_pose.pose(), &pose));
}

#[test]
fn render_scene_skeletal_pose_change_stays_in_deformation_and_bounds_domains() {
    let mut scene = test_scene();
    scene
        .apply_delta(RenderSceneDelta::new(vec![test_primitive(82)], Vec::new()))
        .expect("initial add");
    let changed = test_primitive_with(82, |descriptor| {
        descriptor.skeletal_pose = Some(test_skeletal_pose(Transform {
            translation: Vec3::new(0.25, 0.5, 0.75),
            ..Transform::default()
        }));
    });

    let journal = scene
        .apply_delta(RenderSceneDelta::new(vec![changed], Vec::new()))
        .expect("skeletal pose update");
    let update = &journal.updates()[0];
    let dirty = update.dirty();

    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::DEFORMATION));
    assert!(dirty.contains(RenderScenePrimitiveDirtyFlags::BOUNDS));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::LOCAL_BOUNDS));
    assert_eq!(journal.stats().dirty_domain_counts().total_count(), 2);
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::TRANSFORM));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::GEOMETRY));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::MATERIAL));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::RENDER_STATE));
    assert!(!dirty.contains(RenderScenePrimitiveDirtyFlags::VISIBILITY));
    let pose = update
        .primitive()
        .descriptor()
        .skeletal_pose
        .as_ref()
        .expect("journal retains skeletal pose input");
    assert_eq!(
        pose.skeleton(),
        &ResourceId::from_stable_label("tests/render-scene/skeleton")
    );
    assert_eq!(pose.pose().bones[0].local_transform.translation.x, 0.25);
}

#[test]
fn render_scene_primitive_rejects_non_finite_skeletal_pose_transforms() {
    for (transform, expected_field) in [
        (
            Transform {
                translation: Vec3::new(f32::NAN, 0.0, 0.0),
                ..Transform::default()
            },
            RenderScenePrimitiveField::SkeletalPoseTranslation,
        ),
        (
            Transform {
                rotation: Quat::from_xyzw(0.0, 0.0, 0.0, f32::INFINITY),
                ..Transform::default()
            },
            RenderScenePrimitiveField::SkeletalPoseRotation,
        ),
        (
            Transform {
                scale: Vec3::new(1.0, f32::NAN, 1.0),
                ..Transform::default()
            },
            RenderScenePrimitiveField::SkeletalPoseScale,
        ),
    ] {
        let mut descriptor = test_descriptor(83, stable_key(83));
        descriptor.skeletal_pose = Some(test_skeletal_pose(transform));

        let error = RenderScenePrimitive::new(
            descriptor,
            RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
                [-1.0; 3], [1.0; 3],
            )),
            test_revisions(1, 1, 1, 1, 1),
        )
        .expect_err("non-finite skeletal pose input must fail");

        assert_eq!(error.stable_instance_key(), stable_key(83));
        assert_eq!(error.field(), expected_field);
    }
}

fn test_skeletal_pose(local_transform: Transform) -> RenderSceneSkeletalPose {
    RenderSceneSkeletalPose::new(
        ResourceId::from_stable_label("tests/render-scene/skeleton"),
        Arc::new(AnimationPoseOutput {
            source: AnimationPoseSource::Clip,
            active_state: None,
            bones: vec![AnimationPoseBone {
                name: "root".to_owned(),
                local_transform,
            }],
        }),
    )
}
