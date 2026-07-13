use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::{AnimationParameterMap, AnimationPoseOutput};
use zircon_runtime::scene::{AnimationStateTransitionRuntime, EntityId};

use super::machine_instance_key::MachineInstanceKey;
use super::nested_machine_resolve::resolve_machine_instance;
use super::nested_machine_sample::{
    normalized_machine_state_time, sample_machine_state_events, sample_machine_state_pose,
    sample_machine_transition_pose,
};
use super::requests::PendingStateMachinePoseSample;
use super::state_machine_transition::{
    advance_state_machine_transition, begin_state_machine_transition, select_interruption_candidate,
};
use super::AnimationEvaluationPipeline;
use crate::CompiledAnimationStateMachine;

pub(super) fn resolve_state_machine_pose_requests(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingStateMachinePoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<crate::AnimationClipEvent>,
    Vec<(EntityId, Option<String>)>,
    BTreeMap<EntityId, AnimationStateTransitionRuntime>,
) {
    let mut poses = BTreeMap::new();
    let mut events = Vec::new();
    let mut active_state_updates = Vec::new();
    let mut transition_updates = BTreeMap::new();
    pipeline.retain_interrupted_transition_sources(
        &pending_samples
            .iter()
            .map(|pending| pending.entity)
            .collect::<BTreeSet<_>>(),
    );
    pipeline.retain_nested_machine_instances(
        &pending_samples
            .iter()
            .map(|pending| pending.entity)
            .collect::<BTreeSet<_>>(),
    );

    for pending in pending_samples {
        let instance = MachineInstanceKey::root(pending.entity, pending.state_machine_id);
        let Some(mut resolved) = resolve_machine_instance(
            pipeline,
            asset_manager,
            instance,
            pending.state_machine_id,
            pending.active_state.clone(),
            pending.transition.clone(),
            &pending.parameters,
            pending.skeleton_id,
            pending.to_time_seconds,
        ) else {
            continue;
        };
        let instance = resolved.instance;
        let state_machine = resolved.machine;
        let evaluation = resolved.evaluation;
        let requested_desc = resolved.requested_desc;
        let root_active_state = resolved.root_active_state;
        let is_nested = resolved.is_nested;
        let Some(active_state) = evaluation.active_state.as_deref() else {
            continue;
        };
        let mut interrupted_source = None;
        let transition = if let Some(previous) = resolved.transition.take() {
            let advanced = advance_state_machine_transition(previous, pending.delta_seconds);
            let candidate = select_interruption_candidate(
                pipeline,
                asset_manager,
                state_machine.as_ref(),
                &evaluation.parameters,
                pending.skeleton_id,
                &advanced,
            );
            if let Some(candidate) = candidate {
                let previous_source = pipeline.interrupted_transition_source(
                    &instance,
                    &advanced.from_state,
                    &advanced.to_state,
                );
                let sampled_source = sample_state_transition_pose(
                    pipeline,
                    asset_manager,
                    state_machine.as_ref(),
                    &evaluation.parameters,
                    &pending,
                    &instance,
                    &advanced,
                    previous_source.as_deref(),
                )
                .map(|(_, pose)| pose);
                if let Some(source) = sampled_source {
                    events.extend(sample_state_transition_clip_events(
                        pipeline,
                        asset_manager,
                        state_machine.as_ref(),
                        &evaluation.parameters,
                        &pending,
                        &instance,
                        &advanced,
                    ));
                    interrupted_source = Some(source);
                    Some(begin_state_machine_transition(
                        &candidate.transition,
                        candidate.from_time_seconds,
                        0.0,
                    ))
                } else {
                    Some(advanced)
                }
            } else {
                Some(advanced)
            }
        } else {
            let normalized_state_time = normalized_machine_state_time(
                pipeline,
                asset_manager,
                &instance,
                state_machine.as_ref(),
                active_state,
                &evaluation.parameters,
                pending.skeleton_id,
                pending.to_time_seconds,
            );
            evaluation
                .transition
                .as_ref()
                .zip(requested_desc)
                .filter(|(_, desc)| desc.exit_ready(normalized_state_time))
                .map(|(requested, desc)| {
                    begin_state_machine_transition(
                        requested,
                        pending.to_time_seconds,
                        if desc.exit_time().is_some() {
                            0.0
                        } else {
                            pending.delta_seconds
                        },
                    )
                })
        };
        if let (Some(source), Some(active_transition)) = (interrupted_source, transition.as_ref()) {
            pipeline.record_interrupted_transition_source(
                instance.clone(),
                &active_transition.from_state,
                &active_transition.to_state,
                source,
            );
        }
        let state_update = transition
            .as_ref()
            .map(|transition| {
                if transition.elapsed_seconds >= transition.duration_seconds {
                    transition.to_state.clone()
                } else {
                    transition.from_state.clone()
                }
            })
            .or_else(|| evaluation.active_state.clone());
        if is_nested {
            active_state_updates.push((pending.entity, Some(root_active_state)));
            if let Some(state) = state_update.as_ref() {
                pipeline
                    .nested_machine_states
                    .insert(instance.clone(), state.clone());
            }
        } else {
            active_state_updates.push((pending.entity, state_update.clone()));
        }

        if let Some(active_transition) = transition.as_ref() {
            let active_source = pipeline.interrupted_transition_source(
                &instance,
                &active_transition.from_state,
                &active_transition.to_state,
            );
            events.extend(sample_state_transition_clip_events(
                pipeline,
                asset_manager,
                state_machine.as_ref(),
                &evaluation.parameters,
                &pending,
                &instance,
                active_transition,
            ));
            let Some((entity, pose)) = sample_state_transition_pose(
                pipeline,
                asset_manager,
                state_machine.as_ref(),
                &evaluation.parameters,
                &pending,
                &instance,
                active_transition,
                active_source.as_deref(),
            ) else {
                continue;
            };
            poses.insert(entity, pose);
            if active_transition.elapsed_seconds < active_transition.duration_seconds {
                if is_nested {
                    pipeline
                        .nested_machine_transitions
                        .insert(instance.clone(), active_transition.clone());
                } else {
                    transition_updates.insert(entity, active_transition.clone());
                }
            } else {
                pipeline.clear_interrupted_transition_source(&instance);
                pipeline.nested_machine_transitions.remove(&instance);
            }
            continue;
        }

        pipeline.clear_interrupted_transition_source(&instance);

        let Some(active_state) = state_update.as_deref() else {
            continue;
        };
        events.extend(sample_machine_state_events(
            pipeline,
            asset_manager,
            &instance,
            state_machine.as_ref(),
            active_state,
            &evaluation.parameters,
            pending.entity,
            pending.skeleton_id,
            pending.from_time_seconds,
            pending.to_time_seconds,
        ));
        let Some((entity, pose)) = sample_machine_state_pose(
            pipeline,
            asset_manager,
            &instance,
            state_machine.as_ref(),
            active_state,
            &evaluation.parameters,
            pending.entity,
            pending.skeleton_id,
            pending.to_time_seconds,
        ) else {
            continue;
        };
        poses.insert(entity, pose);
    }

    (poses, events, active_state_updates, transition_updates)
}

fn sample_state_transition_pose(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    state_machine: &CompiledAnimationStateMachine,
    parameters: &AnimationParameterMap,
    pending: &PendingStateMachinePoseSample,
    instance: &MachineInstanceKey,
    transition: &AnimationStateTransitionRuntime,
    interrupted_source: Option<&AnimationPoseOutput>,
) -> Option<(EntityId, AnimationPoseOutput)> {
    sample_machine_transition_pose(
        pipeline,
        asset_manager,
        instance,
        state_machine,
        parameters,
        pending.entity,
        pending.skeleton_id,
        transition,
        interrupted_source,
    )
}

fn sample_state_transition_clip_events(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    state_machine: &CompiledAnimationStateMachine,
    parameters: &AnimationParameterMap,
    pending: &PendingStateMachinePoseSample,
    instance: &MachineInstanceKey,
    transition: &AnimationStateTransitionRuntime,
) -> Vec<crate::AnimationClipEvent> {
    let mut events = Vec::new();
    let (from_start_seconds, to_start_seconds) = pending
        .transition
        .as_ref()
        .filter(|previous| {
            previous.from_state == transition.from_state && previous.to_state == transition.to_state
        })
        .map(|previous| (previous.from_time_seconds, previous.to_time_seconds))
        .unwrap_or((pending.from_time_seconds, 0.0));

    events.extend(sample_machine_state_events(
        pipeline,
        asset_manager,
        instance,
        state_machine,
        &transition.from_state,
        parameters,
        pending.entity,
        pending.skeleton_id,
        from_start_seconds,
        transition.from_time_seconds,
    ));
    events.extend(sample_machine_state_events(
        pipeline,
        asset_manager,
        instance,
        state_machine,
        &transition.to_state,
        parameters,
        pending.entity,
        pending.skeleton_id,
        to_start_seconds,
        transition.to_time_seconds,
    ));
    events
}
