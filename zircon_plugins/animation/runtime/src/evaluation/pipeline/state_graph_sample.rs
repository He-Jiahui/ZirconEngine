use zircon_runtime::animation::sample_clip_events;
use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::{
    AnimationParameterMap, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::EntityId;

use super::clip_sample::sample_pose_request;
use super::graph_evaluate::{sample_compiled_graph_clip_events, sample_compiled_graph_pose};
use super::pose_blend::blend_weighted_poses;
use super::requests::PendingPoseSample;
use super::AnimationEvaluationPipeline;
use crate::state_machine::CompiledGraphSamples;
use crate::CompiledAnimationStateMachine;

pub(super) fn normalized_graph_time(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    graph_samples: CompiledGraphSamples<'_>,
    parameters: &AnimationParameterMap,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
) -> Real {
    let mut duration: Real = 0.0;
    for (graph_reference, weight) in graph_samples.into_iter().flatten() {
        if weight <= 0.0 {
            continue;
        }
        let Some(graph_id) = asset_manager.resolve_asset_id(&graph_reference.locator) else {
            continue;
        };
        let Some(graph) = pipeline.evaluate_graph(asset_manager, graph_id, skeleton_id, parameters)
        else {
            continue;
        };
        duration = duration.max(
            pipeline
                .graph_duration_seconds(asset_manager, graph_id, skeleton_id, &graph)
                .unwrap_or(0.0),
        );
    }
    if duration <= Real::EPSILON || !time_seconds.is_finite() {
        return 1.0;
    }
    (time_seconds.max(0.0) / duration).clamp(0.0, 1.0)
}

pub(super) fn normalized_state_time(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    state_machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: &AnimationParameterMap,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
) -> Real {
    if let Some(clip) = state_machine.clip_for_state(state_name) {
        let duration = asset_manager
            .resolve_asset_id(&clip.locator)
            .and_then(|clip_id| asset_manager.load_animation_clip_asset(clip_id).ok())
            .map(|clip| clip.duration_seconds)
            .filter(|duration| duration.is_finite() && *duration > Real::EPSILON);
        return duration
            .map(|duration| (time_seconds.max(0.0) / duration).clamp(0.0, 1.0))
            .unwrap_or(1.0);
    }
    let samples = state_machine
        .graph_samples_for_state(state_name, parameters)
        .unwrap_or([None, None, None]);
    normalized_graph_time(
        pipeline,
        asset_manager,
        samples,
        parameters,
        skeleton_id,
        time_seconds,
    )
}

pub(super) fn sample_state_clip_events(
    asset_manager: &ProjectAssetManager,
    state_machine: &CompiledAnimationStateMachine,
    state_name: &str,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
) -> Vec<crate::AnimationClipEvent> {
    let Some(clip) = state_machine.clip_for_state(state_name) else {
        return Vec::new();
    };
    let Some(clip_id) = asset_manager.resolve_asset_id(&clip.locator) else {
        return Vec::new();
    };
    let Ok(clip) = asset_manager.load_animation_clip_asset(clip_id) else {
        return Vec::new();
    };
    sample_clip_events(&clip, entity, from_time_seconds, to_time_seconds, false)
}

pub(super) fn sample_state_clip_pose(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    state_machine: &CompiledAnimationStateMachine,
    state_name: &str,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let clip = state_machine.clip_for_state(state_name)?;
    let clip_id = asset_manager.resolve_asset_id(&clip.locator)?;
    sample_pose_request(
        pipeline.clip_evaluator_mut(),
        asset_manager,
        PendingPoseSample {
            entity,
            skeleton_id,
            clip_id,
            time_seconds,
            looping: false,
            source: AnimationPoseSource::StateMachine,
            active_state: Some(state_name.to_string()),
        },
    )
}

pub(super) fn sample_state_events(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    state_machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    from_time_seconds: Real,
    to_time_seconds: Real,
) -> Vec<crate::AnimationClipEvent> {
    if state_machine.clip_for_state(state_name).is_some() {
        return sample_state_clip_events(
            asset_manager,
            state_machine,
            state_name,
            entity,
            from_time_seconds,
            to_time_seconds,
        );
    }
    let samples = state_machine
        .graph_samples_for_state(state_name, parameters)
        .unwrap_or([None, None, None]);
    sample_state_graph_clip_events(
        pipeline,
        asset_manager,
        samples,
        parameters,
        entity,
        skeleton_id,
        from_time_seconds,
        to_time_seconds,
    )
}

pub(super) fn sample_state_pose(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    state_machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
) -> Option<(EntityId, AnimationPoseOutput)> {
    if state_machine.clip_for_state(state_name).is_some() {
        return sample_state_clip_pose(
            pipeline,
            asset_manager,
            state_machine,
            state_name,
            entity,
            skeleton_id,
            time_seconds,
        );
    }
    let samples = state_machine.graph_samples_for_state(state_name, parameters)?;
    sample_state_graph_pose(
        pipeline,
        asset_manager,
        samples,
        parameters,
        entity,
        skeleton_id,
        time_seconds,
        Some(state_name.to_string()),
    )
}

pub(super) fn sample_state_graph_clip_events(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    graph_samples: CompiledGraphSamples<'_>,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    from_time_seconds: Real,
    to_time_seconds: Real,
) -> Vec<crate::AnimationClipEvent> {
    let mut events = Vec::new();
    for (graph_reference, weight) in graph_samples.into_iter().flatten() {
        if weight <= 0.0 {
            continue;
        }
        let Some(graph_id) = asset_manager.resolve_asset_id(&graph_reference.locator) else {
            continue;
        };
        let Some(evaluation) =
            pipeline.evaluate_graph(asset_manager, graph_id, skeleton_id, parameters)
        else {
            continue;
        };
        events.extend(sample_compiled_graph_clip_events(
            asset_manager,
            entity,
            from_time_seconds,
            to_time_seconds,
            &evaluation,
        ));
    }
    events
}

pub(super) fn sample_state_graph_pose(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    graph_samples: CompiledGraphSamples<'_>,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
    active_state: Option<String>,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let mut weighted_poses = Vec::new();
    for (graph_reference, weight) in graph_samples.into_iter().flatten() {
        if weight <= 0.0 {
            continue;
        }
        let graph_id = asset_manager.resolve_asset_id(&graph_reference.locator)?;
        let graph_evaluation =
            pipeline.evaluate_graph(asset_manager, graph_id, skeleton_id, parameters)?;
        let (_, pose) = sample_compiled_graph_pose(
            pipeline.clip_evaluator_mut(),
            asset_manager,
            entity,
            skeleton_id,
            time_seconds,
            AnimationPoseSource::StateMachine,
            active_state.clone(),
            &graph_evaluation,
        )?;
        weighted_poses.push((pose, weight));
    }
    if weighted_poses.len() == 1 {
        return weighted_poses.pop().map(|(pose, _)| (entity, pose));
    }
    blend_weighted_poses(
        weighted_poses,
        AnimationPoseSource::StateMachine,
        active_state,
    )
    .map(|pose| (entity, pose))
}
