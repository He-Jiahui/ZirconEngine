//! 图及状态机输出按骨架行做确定性基础混合，再将加法姿态转成相对绑定姿态的增量。
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
    mut weighted_poses: Vec<GraphWeightedPose>,
    source: AnimationPoseSource,
    active_state: Option<String>,
) -> Option<AnimationPoseOutput> {
    let first = weighted_poses.first_mut()?;
    let first_weight = first.weight;
    let first_target_mask = first.target_mask.take();
    let first_target_ids = std::mem::take(&mut first.legacy_target_ids);
    let mut bones = std::mem::take(&mut first.pose.bones);
    for (bone_index, bone) in bones.iter_mut().enumerate() {
        let first_targets_bone = graph_pose_targets_bone(
            first_target_mask.as_deref(),
            &first_target_ids,
            bone_index,
            bone,
        );
        let mut total_weight = if first_targets_bone {
            finite_positive_weight(first_weight).unwrap_or(0.0) as f64
        } else {
            0.0
        };
        for weighted in weighted_poses.iter().skip(1) {
            if graph_pose_targets_bone(
                weighted.target_mask.as_deref(),
                &weighted.legacy_target_ids,
                bone_index,
                bone,
            ) && weighted.pose.bones.get(bone_index).is_some()
            {
                total_weight += finite_positive_weight(weighted.weight).unwrap_or(0.0) as f64;
            }
        }
        if total_weight <= f64::EPSILON {
            continue;
        }

        let mut translation = Vec3::ZERO;
        let mut scale = Vec3::ZERO;
        let mut rotation = Quat::from_xyzw(0.0, 0.0, 0.0, 0.0);
        if first_targets_bone {
            if let Some(weight) = finite_positive_weight(first_weight) {
                accumulate_base_transform(
                    &mut translation,
                    &mut rotation,
                    &mut scale,
                    bone.local_transform,
                    (f64::from(weight) / total_weight) as Real,
                );
            }
        }
        for weighted in weighted_poses.iter().skip(1) {
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
            let Some(weight) = finite_positive_weight(weighted.weight) else {
                continue;
            };
            accumulate_base_transform(
                &mut translation,
                &mut rotation,
                &mut scale,
                other.local_transform,
                (f64::from(weight) / total_weight) as Real,
            );
        }

        bone.local_transform.translation = translation;
        bone.local_transform.rotation = rotation.normalize();
        bone.local_transform.scale = scale;
    }

    Some(AnimationPoseOutput {
        source,
        active_state,
        bones,
    })
}

fn finite_positive_weight(weight: Real) -> Option<Real> {
    (weight.is_finite() && weight > 0.0).then_some(weight)
}

fn accumulate_base_transform(
    translation: &mut Vec3,
    rotation: &mut Quat,
    scale: &mut Vec3,
    transform: zircon_runtime::core::math::Transform,
    normalized_weight: Real,
) {
    *translation += transform.translation * normalized_weight;
    *scale += transform.scale * normalized_weight;
    *rotation += canonical_rotation(transform.rotation) * normalized_weight;
}

fn canonical_rotation(rotation: Quat) -> Quat {
    let components = rotation.to_array();
    let mut canonical_index = 0;
    for index in 1..components.len() {
        if components[index].abs() > components[canonical_index].abs() {
            canonical_index = index;
        }
    }
    if components[canonical_index].is_sign_negative() {
        -rotation
    } else {
        rotation
    }
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

#[cfg(test)]
#[path = "tests/pose_blend.rs"]
mod tests;
