//! 在解析后的子状态机上采样姿态、事件与转换进度；子机时间和根机时间共用同一帧参数投影。
use std::sync::Arc;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::{AnimationPoseOutput, AnimationPoseSource};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::{AnimationStateTransitionRuntime, EntityId};

use super::machine_instance_key::MachineInstanceKey;
use super::pose_blend::blend_weighted_poses;
use super::requests::{PendingClipEventSample, StateMachineParameterProjection};
use super::state_graph_sample::{
    normalized_state_time, sample_state_events, sample_state_pose_with_demand,
};
use super::state_machine_cache::resolve_sub_machine_id;
use super::AnimationEvaluationPipeline;
use crate::{
    CompiledAnimationStateMachine, TransitionDesc, TransitionRequest, TransitionRuntime,
    TransitionState,
};

pub(super) fn normalized_machine_state_time(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: StateMachineParameterProjection<'_>,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
) -> Real {
    let Some((child_instance, child, child_state)) = resolve_child_state(
        pipeline,
        asset_manager,
        instance,
        machine,
        state_name,
        parameters,
    ) else {
        return normalized_state_time(
            pipeline,
            asset_manager,
            instance,
            machine,
            state_name,
            parameters,
            skeleton_id,
            time_seconds,
        );
    };
    normalized_machine_state_time(
        pipeline,
        asset_manager,
        &child_instance,
        &child,
        &child_state,
        parameters,
        skeleton_id,
        time_seconds,
    )
}

pub(super) fn sample_machine_state_pose(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: StateMachineParameterProjection<'_>,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
) -> Option<(EntityId, AnimationPoseOutput)> {
    sample_machine_state_pose_with_demand(
        pipeline,
        asset_manager,
        instance,
        machine,
        state_name,
        parameters,
        entity,
        skeleton_id,
        time_seconds,
        false,
    )
    .map(|(entity, pose, _)| (entity, pose))
}

pub(super) fn sample_machine_state_pose_with_demand(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: StateMachineParameterProjection<'_>,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    time_seconds: Real,
    playing: bool,
) -> Option<(EntityId, AnimationPoseOutput, bool)> {
    let Some((child_instance, child, child_state)) = resolve_child_state(
        pipeline,
        asset_manager,
        instance,
        machine,
        state_name,
        parameters,
    ) else {
        return sample_state_pose_with_demand(
            pipeline,
            asset_manager,
            instance,
            machine,
            state_name,
            parameters,
            entity,
            skeleton_id,
            time_seconds,
            playing,
        );
    };
    let transition = pipeline
        .nested_machine_transitions
        .get(&child_instance)
        .cloned();
    if let Some(transition) = transition {
        return sample_machine_transition_pose_with_demand(
            pipeline,
            asset_manager,
            &child_instance,
            &child,
            parameters,
            entity,
            skeleton_id,
            &transition,
            None,
            playing,
        );
    }
    sample_machine_state_pose_with_demand(
        pipeline,
        asset_manager,
        &child_instance,
        &child,
        &child_state,
        parameters,
        entity,
        skeleton_id,
        time_seconds,
        playing,
    )
}

pub(super) fn sample_machine_transition_pose(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    machine: &CompiledAnimationStateMachine,
    parameters: StateMachineParameterProjection<'_>,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    transition: &AnimationStateTransitionRuntime,
    interrupted_source: Option<&AnimationPoseOutput>,
) -> Option<(EntityId, AnimationPoseOutput)> {
    sample_machine_transition_pose_with_demand(
        pipeline,
        asset_manager,
        instance,
        machine,
        parameters,
        entity,
        skeleton_id,
        transition,
        interrupted_source,
        false,
    )
    .map(|(entity, pose, _)| (entity, pose))
}

pub(super) fn sample_machine_transition_pose_with_demand(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    machine: &CompiledAnimationStateMachine,
    parameters: StateMachineParameterProjection<'_>,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    transition: &AnimationStateTransitionRuntime,
    interrupted_source: Option<&AnimationPoseOutput>,
    playing: bool,
) -> Option<(EntityId, AnimationPoseOutput, bool)> {
    let (from_pose, from_requires_frame) = interrupted_source
        .cloned()
        .map(|pose| (pose, false))
        .or_else(|| {
            sample_machine_state_pose_with_demand(
                pipeline,
                asset_manager,
                instance,
                machine,
                &transition.from_state,
                parameters,
                entity,
                skeleton_id,
                transition.from_time_seconds,
                playing,
            )
            .map(|(_, pose, requires_frame)| (pose, requires_frame))
        })?;
    let (_, to_pose, to_requires_frame) = sample_machine_state_pose_with_demand(
        pipeline,
        asset_manager,
        instance,
        machine,
        &transition.to_state,
        parameters,
        entity,
        skeleton_id,
        transition.to_time_seconds,
        playing,
    )?;
    let progress = transition_progress(transition);
    let transition_requires_frame = playing
        && transition.elapsed_seconds.is_finite()
        && transition.duration_seconds.is_finite()
        && transition.elapsed_seconds < transition.duration_seconds;
    blend_weighted_poses(
        vec![(from_pose, 1.0 - progress), (to_pose, progress)],
        AnimationPoseSource::StateMachine,
        Some(if progress >= 1.0 {
            transition.to_state.clone()
        } else {
            transition.from_state.clone()
        }),
    )
    .map(|pose| {
        let requires_frame = !pose.bones.is_empty()
            && (transition_requires_frame
                || (from_requires_frame && progress < 1.0)
                || (to_requires_frame && progress > 0.0));
        (entity, pose, requires_frame)
    })
}

pub(super) fn sample_machine_state_events(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: StateMachineParameterProjection<'_>,
    entity: EntityId,
    skeleton_id: zircon_runtime::asset::AssetId,
    from_time_seconds: Real,
    to_time_seconds: Real,
) -> Vec<PendingClipEventSample> {
    let Some((child_instance, child, child_state)) = resolve_child_state(
        pipeline,
        asset_manager,
        instance,
        machine,
        state_name,
        parameters,
    ) else {
        return sample_state_events(
            pipeline,
            asset_manager,
            instance,
            machine,
            state_name,
            parameters,
            entity,
            skeleton_id,
            from_time_seconds,
            to_time_seconds,
        );
    };
    let transition = pipeline
        .nested_machine_transitions
        .get(&child_instance)
        .cloned();
    if let Some(transition) = transition {
        let mut events = sample_machine_state_events(
            pipeline,
            asset_manager,
            &child_instance,
            &child,
            &transition.from_state,
            parameters,
            entity,
            skeleton_id,
            from_time_seconds,
            to_time_seconds,
        );
        events.extend(sample_machine_state_events(
            pipeline,
            asset_manager,
            &child_instance,
            &child,
            &transition.to_state,
            parameters,
            entity,
            skeleton_id,
            from_time_seconds,
            to_time_seconds,
        ));
        return events;
    }
    sample_machine_state_events(
        pipeline,
        asset_manager,
        &child_instance,
        &child,
        &child_state,
        parameters,
        entity,
        skeleton_id,
        from_time_seconds,
        to_time_seconds,
    )
}

fn resolve_child_state(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    machine: &CompiledAnimationStateMachine,
    state_name: &str,
    parameters: StateMachineParameterProjection<'_>,
) -> Option<(
    MachineInstanceKey,
    Arc<CompiledAnimationStateMachine>,
    String,
)> {
    let nested = machine.sub_machine_for_state(state_name)?;
    let nested_id = resolve_sub_machine_id(asset_manager, nested)?;
    let child_instance = instance.nested(state_name, nested_id)?;
    let active_state = pipeline.nested_machine_states.get(&child_instance).cloned();
    let (child, evaluation, _) = pipeline.evaluate_state_machine(
        asset_manager,
        &child_instance,
        nested_id,
        active_state.as_deref(),
        parameters,
    )?;
    Some((child_instance, child, evaluation.active_state))
}

fn transition_progress(transition: &AnimationStateTransitionRuntime) -> Real {
    TransitionRuntime::begin(
        TransitionRequest::new(
            TransitionState::new(0),
            TransitionState::new(1),
            TransitionDesc::new(transition.duration_seconds),
        ),
        transition.elapsed_seconds,
    )
    .crossfade_weights()
    .target
}
