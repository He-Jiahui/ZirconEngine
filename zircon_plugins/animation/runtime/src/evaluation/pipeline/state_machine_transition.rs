use std::sync::Arc;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::AnimationStateTransitionEvaluation;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::AnimationStateTransitionRuntime;

use super::machine_instance_key::MachineInstanceKey;
use super::requests::StateMachineParameterProjection;
use super::state_graph_sample::normalized_state_time;
use super::AnimationEvaluationPipeline;
use crate::{CompiledAnimationStateMachine, TransitionRequest, TransitionRuntime};

pub(super) struct InterruptionCandidate {
    pub(super) transition: AnimationStateTransitionEvaluation,
    pub(super) from_time_seconds: Real,
    pub(super) consumed_triggers: Option<Arc<[String]>>,
}

pub(super) fn advance_state_machine_transition(
    mut previous: AnimationStateTransitionRuntime,
    delta_seconds: Real,
) -> AnimationStateTransitionRuntime {
    let delta_seconds = finite_non_negative(delta_seconds);
    if !previous.duration_seconds.is_finite() || previous.duration_seconds <= Real::EPSILON {
        previous.duration_seconds = 0.0;
        previous.elapsed_seconds = 0.0;
        previous.to_time_seconds = (previous.to_time_seconds + delta_seconds).max(0.0);
        return previous;
    }
    previous.elapsed_seconds = (previous.elapsed_seconds + delta_seconds)
        .min(previous.duration_seconds)
        .max(0.0);
    previous.from_time_seconds = (previous.from_time_seconds + delta_seconds).max(0.0);
    previous.to_time_seconds = (previous.to_time_seconds + delta_seconds).max(0.0);
    previous
}

pub(super) fn begin_state_machine_transition(
    requested: &AnimationStateTransitionEvaluation,
    from_time_seconds: Real,
    elapsed_seconds: Real,
) -> AnimationStateTransitionRuntime {
    AnimationStateTransitionRuntime {
        from_state: requested.from_state.clone(),
        to_state: requested.to_state.clone(),
        duration_seconds: requested.duration_seconds,
        elapsed_seconds: elapsed_seconds.min(requested.duration_seconds).max(0.0),
        from_time_seconds: from_time_seconds.max(0.0),
        to_time_seconds: elapsed_seconds.max(0.0),
    }
}

pub(super) fn select_interruption_candidate(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    instance: &MachineInstanceKey,
    state_machine: &CompiledAnimationStateMachine,
    parameters: StateMachineParameterProjection<'_>,
    skeleton_id: zircon_runtime::asset::AssetId,
    active: &AnimationStateTransitionRuntime,
) -> Option<InterruptionCandidate> {
    let active_desc = state_machine.transition_desc(&active.from_state, &active.to_state)?;
    let active_from = state_machine.transition_state(&active.from_state)?;
    let active_to = state_machine.transition_state(&active.to_state)?;
    let active_runtime = TransitionRuntime::begin(
        TransitionRequest::new(active_from, active_to, active_desc),
        active.elapsed_seconds,
    );
    if active_runtime.is_complete() {
        return None;
    }

    for (state_name, state_time_seconds) in candidate_states(active) {
        let requested_from = state_machine.transition_state(state_name)?;
        if !active_runtime.can_interrupt_from(requested_from) {
            continue;
        }
        let evaluated = pipeline.evaluate_compiled_state_machine_with_sampling(
            instance,
            state_machine,
            Some(state_name),
            parameters,
        );
        let Some(requested) = evaluated.transition().cloned() else {
            continue;
        };
        if requested.from_state == active.from_state && requested.to_state == active.to_state {
            continue;
        }
        let Some(desc) = evaluated.transition_desc() else {
            continue;
        };
        let normalized_time = normalized_state_time(
            pipeline,
            asset_manager,
            instance,
            state_machine,
            state_name,
            parameters,
            skeleton_id,
            state_time_seconds,
        );
        if desc.exit_ready(normalized_time) {
            return Some(InterruptionCandidate {
                transition: requested,
                from_time_seconds: state_time_seconds,
                consumed_triggers: evaluated.shared_consumed_triggers(),
            });
        }
    }
    None
}

fn candidate_states(active: &AnimationStateTransitionRuntime) -> [(&str, Real); 2] {
    [
        (active.to_state.as_str(), active.to_time_seconds),
        (active.from_state.as_str(), active.from_time_seconds),
    ]
}

fn finite_non_negative(value: Real) -> Real {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
