use std::collections::BTreeMap;

use zircon_runtime::animation::sample_clip_events;
use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::EntityId;

use super::clip_sample::sample_pose_request;
use super::pose_blend::{
    apply_graph_additive_poses, blend_graph_base_poses, convert_pose_to_reference_delta,
    GraphWeightedPose,
};
use super::requests::{PendingGraphPoseSample, PendingPoseSample};
use crate::{AnimationClipEvaluator, CompiledAnimationGraphEvaluation, CompiledGraphClipInstance};

use super::AnimationEvaluationPipeline;

pub(super) fn resolve_graph_pose_requests(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingGraphPoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<crate::AnimationClipEvent>,
) {
    let mut poses = BTreeMap::new();
    let mut events = Vec::new();
    for pending in pending_samples {
        let Some(evaluation) = pipeline.evaluate_graph(
            asset_manager,
            pending.graph_id,
            pending.skeleton_id,
            &pending.parameters,
        ) else {
            continue;
        };
        events.extend(sample_compiled_graph_clip_events(
            asset_manager,
            pending.entity,
            pending.from_time_seconds,
            pending.to_time_seconds,
            &evaluation,
        ));
        if let Some((entity, pose)) = sample_compiled_graph_pose(
            pipeline.clip_evaluator_mut(),
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

pub(super) fn sample_compiled_graph_clip_events(
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
    evaluation: &CompiledAnimationGraphEvaluation,
) -> Vec<crate::AnimationClipEvent> {
    evaluation
        .clips()
        .iter()
        .filter_map(|clip| {
            let clip_id = asset_manager.resolve_asset_id(&clip.clip().locator)?;
            let clip_asset = asset_manager.load_animation_clip_asset(clip_id).ok()?;
            Some(sample_clip_events(
                &clip_asset,
                entity,
                resolve_graph_clip_time_seconds(from_time_seconds, clip.playback_speed()),
                resolve_graph_clip_time_seconds(to_time_seconds, clip.playback_speed()),
                clip.looping(),
            ))
        })
        .flatten()
        .collect()
}

pub(super) fn sample_compiled_graph_pose(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    skeleton_id: AssetId,
    base_time_seconds: Real,
    source: AnimationPoseSource,
    active_state: Option<String>,
    evaluation: &CompiledAnimationGraphEvaluation,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let total_weight = evaluation
        .clips()
        .iter()
        .filter(|clip| clip.blend_mode() == AnimationGraphBlendMode::Base)
        .filter_map(finite_positive_compiled_clip_weight)
        .sum::<Real>();
    if total_weight <= Real::EPSILON {
        return None;
    }

    let mut base_poses = Vec::new();
    let mut additive_poses = Vec::new();
    for clip in evaluation.clips() {
        let Some(weight) = finite_positive_compiled_clip_weight(clip) else {
            continue;
        };
        let normalized_weight = match clip.blend_mode() {
            AnimationGraphBlendMode::Base => weight / total_weight,
            AnimationGraphBlendMode::Additive => weight,
        };
        let clip_id = asset_manager.resolve_asset_id(&clip.clip().locator)?;
        let (_, mut pose) = sample_pose_request(
            evaluator,
            asset_manager,
            PendingPoseSample {
                entity,
                skeleton_id,
                clip_id,
                time_seconds: resolve_graph_clip_time_seconds(
                    base_time_seconds,
                    clip.playback_speed(),
                ),
                looping: clip.looping(),
                source,
                active_state: active_state.clone(),
            },
        )?;
        if clip.blend_mode() == AnimationGraphBlendMode::Additive {
            convert_pose_to_reference_delta(&mut pose, evaluator, skeleton_id)?;
        }
        let weighted = GraphWeightedPose {
            pose,
            weight: normalized_weight,
            target_mask: clip.target_mask_owner(),
            legacy_target_ids: Vec::new(),
        };
        match clip.blend_mode() {
            AnimationGraphBlendMode::Base => base_poses.push(weighted),
            AnimationGraphBlendMode::Additive => additive_poses.push(weighted),
        }
    }

    let mut pose = blend_graph_base_poses(base_poses, source, active_state)?;
    apply_graph_additive_poses(&mut pose, additive_poses);
    Some((entity, pose))
}

fn resolve_graph_clip_time_seconds(base_time_seconds: Real, playback_speed: Real) -> Real {
    (base_time_seconds * playback_speed).max(0.0)
}

fn finite_positive_compiled_clip_weight(clip: &CompiledGraphClipInstance) -> Option<Real> {
    (clip.weight().is_finite() && clip.weight() > 0.0).then_some(clip.weight())
}
