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
mod tests {
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
}
