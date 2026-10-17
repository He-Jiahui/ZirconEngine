//! 从编译图贡献分别产生姿态请求和剪辑事件范围；基础姿态归一混合后再应用相对绑定姿态的加法层。
use std::collections::BTreeMap;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::EntityId;

use super::clip_sample::sample_pose_request_with_demand;
use super::pose_blend::{
    apply_graph_additive_poses, blend_graph_base_poses, convert_pose_to_reference_delta,
    GraphWeightedPose,
};
use super::requests::{PendingClipEventSample, PendingGraphPoseSample, PendingPoseSample};
use crate::{AnimationClipEvaluator, CompiledAnimationGraphEvaluation, CompiledGraphClipInstance};

use super::AnimationEvaluationPipeline;

pub(super) fn resolve_graph_pose_requests(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingGraphPoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<PendingClipEventSample>,
) {
    let (poses, events, _) =
        resolve_graph_pose_requests_with_demand(pipeline, asset_manager, pending_samples);
    (poses, events)
}

pub(super) fn resolve_graph_pose_requests_with_demand(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingGraphPoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<PendingClipEventSample>,
    bool,
) {
    let mut poses = BTreeMap::new();
    let mut events = Vec::new();
    let mut requires_continuous_frame = false;
    for pending in pending_samples {
        let Some(evaluation) = pipeline.evaluate_graph(
            asset_manager,
            pending.graph_id,
            pending.skeleton_id,
            &pending.parameters,
        ) else {
            continue;
        };
        events.extend(sample_compiled_graph_clip_event_samples(
            asset_manager,
            pending.entity,
            pending.from_time_seconds,
            pending.to_time_seconds,
            &evaluation,
        ));
        if let Some((entity, pose, requires_frame)) = sample_compiled_graph_pose_with_demand(
            pipeline.clip_evaluator_mut(),
            asset_manager,
            pending.entity,
            pending.skeleton_id,
            pending.to_time_seconds,
            AnimationPoseSource::Graph,
            None,
            &evaluation,
            pending.playing,
        ) {
            poses.insert(entity, pose);
            requires_continuous_frame |= requires_frame;
        }
    }
    (poses, events, requires_continuous_frame)
}

// TODO: [CR-PLUGIN-ANIMATION-0004] 确认零权重剪辑是否仍应发送轨道事件；姿态采样会跳过零权重，事件采样却遍历全部贡献；下一步补充混合图事件契约测试。
pub(super) fn sample_compiled_graph_clip_event_samples(
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
    evaluation: &CompiledAnimationGraphEvaluation,
) -> Vec<PendingClipEventSample> {
    evaluation
        .clips()
        .iter()
        .filter_map(|clip| {
            let clip_id = asset_manager.resolve_asset_id(&clip.clip().locator)?;
            Some(PendingClipEventSample {
                entity,
                clip_id,
                from_time_seconds: resolve_graph_clip_time_seconds(
                    from_time_seconds,
                    clip.playback_speed(),
                ),
                to_time_seconds: resolve_graph_clip_time_seconds(
                    to_time_seconds,
                    clip.playback_speed(),
                ),
                looping: clip.looping(),
            })
        })
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
    sample_compiled_graph_pose_with_demand(
        evaluator,
        asset_manager,
        entity,
        skeleton_id,
        base_time_seconds,
        source,
        active_state,
        evaluation,
        false,
    )
    .map(|(entity, pose, _)| (entity, pose))
}

pub(super) fn sample_compiled_graph_pose_with_demand(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    skeleton_id: AssetId,
    base_time_seconds: Real,
    source: AnimationPoseSource,
    active_state: Option<String>,
    evaluation: &CompiledAnimationGraphEvaluation,
    playing: bool,
) -> Option<(EntityId, AnimationPoseOutput, bool)> {
    let mut requires_continuous_frame = false;
    let mut base_poses = Vec::new();
    let mut additive_poses = Vec::new();
    for clip in evaluation.clips() {
        let Some(weight) = finite_positive_compiled_clip_weight(clip) else {
            continue;
        };
        let clip_id = asset_manager.resolve_asset_id(&clip.clip().locator)?;
        let (_, mut pose, requires_frame) = sample_pose_request_with_demand(
            evaluator,
            asset_manager,
            PendingPoseSample {
                entity,
                skeleton_id,
                clip_id,
                playing,
                playback_speed: clip.playback_speed(),
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
        let target_mask = clip.target_mask_owner();
        let targets_pose = target_mask
            .as_deref()
            .map_or(!pose.bones.is_empty(), |mask| {
                mask.iter().take(pose.bones.len()).any(|included| *included)
            });
        requires_continuous_frame |= requires_frame && targets_pose;
        let weighted = GraphWeightedPose {
            pose,
            weight,
            target_mask,
            legacy_target_ids: Vec::new(),
        };
        match clip.blend_mode() {
            AnimationGraphBlendMode::Base => base_poses.push(weighted),
            AnimationGraphBlendMode::Additive => additive_poses.push(weighted),
        }
    }

    let mut pose = blend_graph_base_poses(base_poses, source, active_state)?;
    apply_graph_additive_poses(&mut pose, additive_poses);
    Some((entity, pose, requires_continuous_frame))
}

fn resolve_graph_clip_time_seconds(base_time_seconds: Real, playback_speed: Real) -> Real {
    (base_time_seconds * playback_speed).max(0.0)
}

fn finite_positive_compiled_clip_weight(clip: &CompiledGraphClipInstance) -> Option<Real> {
    (clip.weight().is_finite() && clip.weight() > 0.0).then_some(clip.weight())
}
