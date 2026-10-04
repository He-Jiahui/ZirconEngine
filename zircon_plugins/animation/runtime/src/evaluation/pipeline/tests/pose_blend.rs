use std::sync::Arc;

use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Transform, Vec3};

use super::{blend_graph_base_poses, GraphWeightedPose};

#[test]
fn masked_base_blend_normalizes_weights_per_bone() {
    let first = pose_with_translations(10.0, 20.0);
    let masked = pose_with_translations(100.0, 40.0);

    let blended = blend_graph_base_poses(
        vec![
            weighted(first, 0.5, None),
            weighted(masked, 0.5, Some(Arc::from([false, true]))),
        ],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("base poses should blend");

    assert_eq!(blended.bones[0].local_transform.translation.x, 10.0);
    assert_eq!(blended.bones[1].local_transform.translation.x, 30.0);
}

#[test]
fn base_blend_ignores_non_positive_and_non_finite_weights() {
    let blended = blend_graph_base_poses(
        vec![
            weighted(pose_with_translations(10.0, 20.0), 0.5, None),
            weighted(pose_with_translations(100.0, 200.0), f32::NAN, None),
            weighted(pose_with_translations(300.0, 400.0), -1.0, None),
            weighted(pose_with_translations(500.0, 600.0), f32::INFINITY, None),
        ],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("one valid base pose should blend");

    assert_eq!(blended.bones[0].local_transform.translation.x, 10.0);
    assert_eq!(blended.bones[1].local_transform.translation.x, 20.0);
}

#[test]
fn overlapping_base_blend_is_deterministic_across_input_order() {
    let left = weighted(pose_with_translations(10.0, 20.0), 0.25, None);
    let right = weighted(pose_with_translations(30.0, 40.0), 0.75, None);

    let forward = blend_graph_base_poses(
        vec![left.clone(), right.clone()],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("forward base poses should blend");
    let reverse = blend_graph_base_poses(vec![right, left], AnimationPoseSource::Graph, None)
        .expect("reverse base poses should blend");

    assert_eq!(forward, reverse);
    assert_eq!(forward.bones[0].local_transform.translation.x, 25.0);
    assert_eq!(forward.bones[1].local_transform.translation.x, 35.0);
}

#[test]
fn base_blend_equivalent_quaternion_signs_use_one_canonical_result() {
    let rotation = zircon_runtime::core::math::Quat::from_rotation_y(0.7);
    let mut positive = pose_with_translations(10.0, 20.0);
    positive.bones[0].local_transform.rotation = rotation;
    let mut negative = pose_with_translations(10.0, 20.0);
    negative.bones[0].local_transform.rotation = -rotation;

    let forward = blend_graph_base_poses(
        vec![
            weighted(positive.clone(), 0.5, None),
            weighted(negative.clone(), 0.5, None),
        ],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("signed quaternion pair should blend");
    let reverse = blend_graph_base_poses(
        vec![weighted(negative, 0.5, None), weighted(positive, 0.5, None)],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("reversed signed quaternion pair should blend");

    assert_eq!(
        forward.bones[0].local_transform.rotation,
        reverse.bones[0].local_transform.rotation
    );
}

fn weighted(
    pose: AnimationPoseOutput,
    weight: f32,
    target_mask: Option<Arc<[bool]>>,
) -> GraphWeightedPose {
    GraphWeightedPose {
        pose,
        weight,
        target_mask,
        legacy_target_ids: Vec::new(),
    }
}

fn pose_with_translations(leg_x: f32, arm_x: f32) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Graph,
        active_state: None,
        bones: vec![bone("leg", leg_x), bone("arm", arm_x)],
    }
}

fn bone(name: &str, translation_x: f32) -> AnimationPoseBone {
    AnimationPoseBone {
        name: name.to_string(),
        local_transform: Transform {
            translation: Vec3::new(translation_x, 0.0, 0.0),
            ..Transform::default()
        },
    }
}
