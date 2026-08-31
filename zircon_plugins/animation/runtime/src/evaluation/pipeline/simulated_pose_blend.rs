use std::collections::BTreeMap;

use zircon_runtime::asset::AssetId;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::framework::physics::SimulatedPoseFeed;
use zircon_runtime::core::math::{Real, Transform};
use zircon_runtime::scene::EntityId;

use super::AnimationEvaluationPipeline;

pub(super) fn blend_simulated_pose_feed(
    pipeline: &AnimationEvaluationPipeline,
    feed: &SimulatedPoseFeed,
    skeletons_by_entity: &BTreeMap<EntityId, AssetId>,
    poses: &mut BTreeMap<EntityId, AnimationPoseOutput>,
) {
    for (entity, pose) in poses {
        let Some(simulated_targets) = feed.targets(*entity) else {
            continue;
        };
        let Some(skeleton_id) = skeletons_by_entity.get(entity).copied() else {
            continue;
        };
        let Some(targets) = pipeline.skeleton_target_table(skeleton_id) else {
            continue;
        };
        for simulated in simulated_targets.iter() {
            let Some(index) = targets.bone_index_for_unique_name(&simulated.bone_name) else {
                continue;
            };
            let Some(bone) = pose.bones.get_mut(index) else {
                continue;
            };
            if bone.name != simulated.bone_name
                || !valid_transform(simulated.local_transform)
                || !valid_weight(simulated.normalized_weight)
            {
                continue;
            }
            let weight = simulated.normalized_weight;
            bone.local_transform =
                blend_simulated_transform(bone.local_transform, simulated.local_transform, weight);
        }
    }
}

fn blend_simulated_transform(current: Transform, simulated: Transform, weight: Real) -> Transform {
    if weight == 0.0 {
        return Transform {
            rotation: current.rotation.normalize(),
            ..current
        };
    }

    let simulated_rotation = simulated.rotation.normalize();
    if weight == 1.0 {
        let rotation = if current.rotation.dot(simulated_rotation) < 0.0 {
            -simulated_rotation
        } else {
            simulated_rotation
        };
        return Transform {
            rotation,
            ..simulated
        };
    }

    Transform {
        translation: current.translation.lerp(simulated.translation, weight),
        rotation: current
            .rotation
            .slerp(simulated_rotation, weight)
            .normalize(),
        scale: current.scale.lerp(simulated.scale, weight),
    }
}

fn valid_transform(transform: Transform) -> bool {
    transform.translation.is_finite()
        && transform.rotation.is_finite()
        && transform.rotation.length_squared() > Real::EPSILON
        && transform.scale.is_finite()
}

fn valid_weight(weight: Real) -> bool {
    weight.is_finite() && (0.0..=1.0).contains(&weight)
}

#[cfg(test)]
#[path = "simulated_pose_blend/performance_tests.rs"]
mod optimization_batch_20260830cw_tests;
