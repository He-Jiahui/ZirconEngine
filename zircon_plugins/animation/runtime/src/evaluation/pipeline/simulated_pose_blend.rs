use std::collections::BTreeMap;

use zircon_runtime::asset::AssetId;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::framework::physics::SimulatedPoseFeed;
use zircon_runtime::core::math::Real;
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
            bone.local_transform.translation = bone
                .local_transform
                .translation
                .lerp(simulated.local_transform.translation, weight);
            bone.local_transform.rotation = bone
                .local_transform
                .rotation
                .slerp(simulated.local_transform.rotation.normalize(), weight)
                .normalize();
            bone.local_transform.scale = bone
                .local_transform
                .scale
                .lerp(simulated.local_transform.scale, weight);
        }
    }
}

fn valid_transform(transform: zircon_runtime::core::math::Transform) -> bool {
    transform.translation.is_finite()
        && transform.rotation.is_finite()
        && transform.rotation.length_squared() > Real::EPSILON
        && transform.scale.is_finite()
}

fn valid_weight(weight: Real) -> bool {
    weight.is_finite() && (0.0..=1.0).contains(&weight)
}
