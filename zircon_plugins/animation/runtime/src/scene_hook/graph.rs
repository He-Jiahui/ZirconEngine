use std::collections::BTreeMap;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationGraphClipInstance, AnimationGraphEvaluation,
    AnimationManager, AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Quat, Real, Vec3};
use zircon_runtime::scene::EntityId;

use crate::clip_event::sample_clip_events;

use super::pending::{PendingGraphPoseSample, PendingPoseSample};
use super::pose::sample_pose_request;

pub(super) fn resolve_graph_pose_requests(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingGraphPoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<crate::AnimationClipEvent>,
) {
    let mut poses = BTreeMap::new();
    let mut events = Vec::new();
    for pending in pending_samples {
        let Some(graph) = asset_manager
            .load_animation_graph_asset(pending.graph_id)
            .ok()
        else {
            continue;
        };
        let evaluation = animation.evaluate_graph(&graph, &pending.parameters);
        events.extend(sample_graph_evaluation_clip_events(
            asset_manager,
            pending.entity,
            pending.from_time_seconds,
            pending.to_time_seconds,
            &evaluation,
        ));
        if let Some((entity, pose)) = sample_graph_evaluation_pose(
            animation,
            asset_manager,
            pending.entity,
            pending.skeleton_id,
            pending.to_time_seconds,
            AnimationPoseSource::Graph,
            None,
            &evaluation,
        ) {
            poses.insert(entity, pose);
        }
    }
    (poses, events)
}

pub(super) fn sample_graph_evaluation_clip_events(
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
    evaluation: &AnimationGraphEvaluation,
) -> Vec<crate::AnimationClipEvent> {
    evaluation
        .clips
        .iter()
        .filter_map(|clip| {
            let clip_id = asset_manager.resolve_asset_id(&clip.clip.locator)?;
            let clip_asset = asset_manager.load_animation_clip_asset(clip_id).ok()?;
            Some(sample_clip_events(
                &clip_asset,
                entity,
                resolve_graph_clip_time_seconds(from_time_seconds, clip.playback_speed),
                resolve_graph_clip_time_seconds(to_time_seconds, clip.playback_speed),
                clip.looping,
            ))
        })
        .flatten()
        .collect()
}

pub(super) fn sample_graph_evaluation_pose(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    skeleton_id: AssetId,
    base_time_seconds: Real,
    source: AnimationPoseSource,
    active_state: Option<String>,
    evaluation: &AnimationGraphEvaluation,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let total_weight = evaluation
        .clips
        .iter()
        .filter(|clip| clip.blend_mode == AnimationGraphBlendMode::Base)
        .filter_map(finite_positive_graph_clip_weight)
        .sum::<Real>();
    if total_weight <= Real::EPSILON {
        return None;
    }

    let mut base_poses = Vec::new();
    let mut additive_poses = Vec::new();
    for clip in &evaluation.clips {
        let Some(weight) = finite_positive_graph_clip_weight(clip) else {
            continue;
        };
        let normalized_weight = match clip.blend_mode {
            AnimationGraphBlendMode::Base => weight / total_weight,
            AnimationGraphBlendMode::Additive => weight,
        };
        let clip_id = asset_manager.resolve_asset_id(&clip.clip.locator)?;
        let (_, pose) = sample_pose_request(
            animation,
            asset_manager,
            PendingPoseSample {
                entity,
                skeleton_id,
                clip_id,
                time_seconds: resolve_graph_clip_time_seconds(
                    base_time_seconds,
                    clip.playback_speed,
                ),
                looping: clip.looping,
                source,
                active_state: active_state.clone(),
            },
        )?;
        match clip.blend_mode {
            AnimationGraphBlendMode::Base => base_poses.push(GraphWeightedPose {
                pose,
                weight: normalized_weight,
                target_ids: clip.target_ids.clone(),
            }),
            AnimationGraphBlendMode::Additive => additive_poses.push(GraphWeightedPose {
                pose,
                weight: normalized_weight,
                target_ids: clip.target_ids.clone(),
            }),
        }
    }

    let mut pose = blend_graph_base_poses(base_poses, source, active_state)?;
    apply_graph_additive_poses(&mut pose, additive_poses);
    Some((entity, pose))
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
                target_ids: Vec::new(),
            })
            .collect(),
        source,
        active_state,
    )
}

fn resolve_graph_clip_time_seconds(base_time_seconds: Real, playback_speed: Real) -> Real {
    (base_time_seconds * playback_speed).max(0.0)
}

#[derive(Clone, Debug)]
struct GraphWeightedPose {
    pose: AnimationPoseOutput,
    weight: Real,
    target_ids: Vec<String>,
}

fn finite_positive_graph_clip_weight(clip: &AnimationGraphClipInstance) -> Option<Real> {
    (clip.weight.is_finite() && clip.weight > 0.0).then_some(clip.weight)
}

fn blend_graph_base_poses(
    weighted_poses: Vec<GraphWeightedPose>,
    source: AnimationPoseSource,
    active_state: Option<String>,
) -> Option<AnimationPoseOutput> {
    let first = weighted_poses.first()?.clone();
    let first_weight = first.weight;
    let first_target_ids = first.target_ids;
    let first_pose = first.pose;
    let mut bones = first_pose.bones;
    for bone in &mut bones {
        if graph_pose_targets_bone(&first_target_ids, bone) {
            bone.local_transform.translation *= first_weight;
            bone.local_transform.scale *= first_weight;
            bone.local_transform.rotation *= first_weight;
        }
    }

    for weighted in weighted_poses.into_iter().skip(1) {
        for bone in &mut bones {
            if !graph_pose_targets_bone(&weighted.target_ids, bone) {
                continue;
            }
            let Some(other) = weighted
                .pose
                .bones
                .iter()
                .find(|other| other.name == bone.name)
            else {
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

fn apply_graph_additive_poses(
    base_pose: &mut AnimationPoseOutput,
    additive_poses: Vec<GraphWeightedPose>,
) {
    for additive in additive_poses {
        for bone in &mut base_pose.bones {
            if !graph_pose_targets_bone(&additive.target_ids, bone) {
                continue;
            }
            let Some(additive_bone) = additive
                .pose
                .bones
                .iter()
                .find(|additive_bone| additive_bone.name == bone.name)
            else {
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

fn graph_pose_targets_bone(target_ids: &[String], bone: &AnimationPoseBone) -> bool {
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
