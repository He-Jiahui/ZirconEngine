use std::sync::Arc;

use zircon_runtime::asset::AssetId;
use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Quat, Real, Vec3};

use crate::AnimationClipEvaluator;

#[derive(Clone, Debug)]
pub(super) struct GraphWeightedPose {
    pub(super) pose: AnimationPoseOutput,
    pub(super) weight: Real,
    pub(super) target_mask: Option<Arc<[bool]>>,
    pub(super) legacy_target_ids: Vec<String>,
}

pub(super) fn convert_pose_to_reference_delta(
    pose: &mut AnimationPoseOutput,
    evaluator: &AnimationClipEvaluator,
    skeleton_id: AssetId,
) -> Option<()> {
    let reference_pose = evaluator.bind_pose(skeleton_id)?;
    if pose.bones.len() != reference_pose.len() {
        return None;
    }
    for (bone, reference) in pose.bones.iter_mut().zip(reference_pose) {
        if bone.name != reference.name {
            return None;
        }
        bone.local_transform.translation -= reference.local_transform.translation;
        bone.local_transform.rotation = (bone.local_transform.rotation
            * reference.local_transform.rotation.inverse())
        .normalize();
        bone.local_transform.scale =
            safe_scale_ratio(bone.local_transform.scale, reference.local_transform.scale);
    }
    Some(())
}

pub(super) fn blend_weighted_poses(
    weighted_poses: Vec<(AnimationPoseOutput, Real)>,
    source: AnimationPoseSource,
    active_state: Option<String>,
) -> Option<AnimationPoseOutput> {
    blend_graph_base_poses(
        weighted_poses
            .into_iter()
            .map(|(pose, weight)| GraphWeightedPose {
                pose,
                weight,
                target_mask: None,
                legacy_target_ids: Vec::new(),
            })
            .collect(),
        source,
        active_state,
    )
}

pub(super) fn blend_graph_base_poses(
    weighted_poses: Vec<GraphWeightedPose>,
    source: AnimationPoseSource,
    active_state: Option<String>,
) -> Option<AnimationPoseOutput> {
    let mut weighted_poses = weighted_poses.into_iter();
    let first = weighted_poses.next()?;
    let first_weight = first.weight;
    let first_target_mask = first.target_mask;
    let first_target_ids = first.legacy_target_ids;
    let first_pose = first.pose;
    let mut bones = first_pose.bones;
    for (bone_index, bone) in bones.iter_mut().enumerate() {
        if graph_pose_targets_bone(
            first_target_mask.as_deref(),
            &first_target_ids,
            bone_index,
            bone,
        ) {
            bone.local_transform.translation *= first_weight;
            bone.local_transform.scale *= first_weight;
            bone.local_transform.rotation *= first_weight;
        }
    }

    for weighted in weighted_poses {
        for (bone_index, bone) in bones.iter_mut().enumerate() {
            if !graph_pose_targets_bone(
                weighted.target_mask.as_deref(),
                &weighted.legacy_target_ids,
                bone_index,
                bone,
            ) {
                continue;
            }
            let Some(other) = weighted.pose.bones.get(bone_index) else {
                continue;
            };
            bone.local_transform.translation += other.local_transform.translation * weighted.weight;
            bone.local_transform.scale += other.local_transform.scale * weighted.weight;
            let mut rotation = other.local_transform.rotation;
            if bone.local_transform.rotation.dot(rotation) < 0.0 {
                rotation = -rotation;
            }
            bone.local_transform.rotation += rotation * weighted.weight;
        }
    }

    for bone in &mut bones {
        bone.local_transform.rotation = bone.local_transform.rotation.normalize();
    }

    Some(AnimationPoseOutput {
        source,
        active_state,
        bones,
    })
}

pub(super) fn apply_graph_additive_poses(
    base_pose: &mut AnimationPoseOutput,
    additive_poses: Vec<GraphWeightedPose>,
) {
    for additive in additive_poses {
        for (bone_index, bone) in base_pose.bones.iter_mut().enumerate() {
            if !graph_pose_targets_bone(
                additive.target_mask.as_deref(),
                &additive.legacy_target_ids,
                bone_index,
                bone,
            ) {
                continue;
            }
            let Some(additive_bone) = additive.pose.bones.get(bone_index) else {
                continue;
            };
            bone.local_transform.translation +=
                additive_bone.local_transform.translation * additive.weight;
            bone.local_transform.scale +=
                (additive_bone.local_transform.scale - Vec3::ONE) * additive.weight;
            let rotation_delta =
                Quat::IDENTITY.slerp(additive_bone.local_transform.rotation, additive.weight);
            bone.local_transform.rotation =
                (rotation_delta * bone.local_transform.rotation).normalize();
        }
    }
}

fn safe_scale_ratio(sample: Vec3, reference: Vec3) -> Vec3 {
    Vec3::new(
        safe_scale_component(sample.x, reference.x),
        safe_scale_component(sample.y, reference.y),
        safe_scale_component(sample.z, reference.z),
    )
}

fn safe_scale_component(sample: Real, reference: Real) -> Real {
    if reference.abs() > Real::EPSILON {
        sample / reference
    } else if sample.abs() <= Real::EPSILON {
        1.0
    } else {
        sample
    }
}

fn graph_pose_targets_bone(
    target_mask: Option<&[bool]>,
    target_ids: &[String],
    bone_index: usize,
    bone: &AnimationPoseBone,
) -> bool {
    if let Some(target_mask) = target_mask {
        return target_mask.get(bone_index).copied().unwrap_or(false);
    }
    target_ids.is_empty()
        || target_ids.iter().any(|target_id| {
            let target_id = target_id.trim();
            target_id == bone.name
                || target_id
                    .rsplit('/')
                    .next()
                    .is_some_and(|leaf| leaf == bone.name)
        })
}
