use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::math::Transform;
use zircon_runtime::scene::{EntityId, LevelSystem};

use super::AnimationEvaluationPipeline;

#[cfg(test)]
#[path = "pose_apply/tests/performance_tests.rs"]
mod optimization_batch_20260830cp_tests;

pub(super) fn apply_pose_transforms_to_scene_nodes(
    level: &LevelSystem,
    replacement_epoch: u64,
    poses: &BTreeMap<EntityId, AnimationPoseOutput>,
) -> Option<usize> {
    level.with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
        let compiled_bindings = {
            let pipeline = world.resource::<AnimationEvaluationPipeline>();
            poses
                .keys()
                .filter(|root| !pipeline.pose_target_binding_is_current(**root, world))
                .filter_map(|root| world.compile_descendant_name_index(*root))
                .collect::<Vec<_>>()
        };
        let updates = {
            let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
            for binding in compiled_bindings {
                pipeline.cache_pose_target_binding(binding);
            }
            node_pose_transform_updates(pipeline, poses)
        };
        let update_count = updates.len();
        for (entity, transform) in updates {
            let _ = world.update_transform(entity, transform);
        }
        update_count
    })
}

fn node_pose_transform_updates(
    pipeline: &AnimationEvaluationPipeline,
    poses: &BTreeMap<EntityId, AnimationPoseOutput>,
) -> Vec<(EntityId, Transform)> {
    let mut updates = Vec::with_capacity(pose_update_capacity(poses.values()));

    for (root, pose) in poses {
        for bone in &pose.bones {
            if let Some(entity) = pipeline.resolve_pose_target(*root, &bone.name) {
                updates.push((entity, bone.local_transform));
            }
        }
    }

    updates
}

fn pose_update_capacity<'a>(poses: impl IntoIterator<Item = &'a AnimationPoseOutput>) -> usize {
    poses
        .into_iter()
        .fold(0, |count, pose| count.saturating_add(pose.bones.len()))
}

#[cfg(test)]
#[path = "tests/pose_apply.rs"]
mod tests;
