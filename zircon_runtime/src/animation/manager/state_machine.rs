use crate::asset::{AnimationConditionOperatorAsset, AnimationStateMachineAsset};
use crate::core::framework::animation::{
    AnimationParameterMap, AnimationParameterValue, AnimationStateMachineEvaluation,
};
use crate::core::math::Real;

use super::parameters::numeric_parameter;
use super::sampling::animation_parameter_value_is_finite;

pub(super) fn evaluate_state_machine(
    state_machine: &AnimationStateMachineAsset,
    current_state: Option<&str>,
    parameters: &AnimationParameterMap,
) -> AnimationStateMachineEvaluation {
    let mut active_state = current_state
        .filter(|state| state_machine_has_state(state_machine, state))
        .map(ToOwned::to_owned)
        .or_else(|| {
            state_machine_has_state(state_machine, &state_machine.entry_state)
                .then(|| state_machine.entry_state.clone())
        });
    let mut transitioned = false;
    let mut transition_evaluation = None;

    if let Some(current) = active_state.as_deref() {
        if let Some(transition) = state_machine.transitions.iter().find(|transition| {
            transition.from_state == current
                && state_machine_has_state(state_machine, &transition.to_state)
                && transition
                    .conditions
                    .iter()
                    .all(|condition| condition_matches(parameters, condition))
        }) {
            let duration_seconds = if transition.duration_seconds.is_finite() {
                transition.duration_seconds.max(0.0)
            } else {
                0.0
            };
            if duration_seconds > Real::EPSILON {
                transition_evaluation = Some(
                    crate::core::framework::animation::AnimationStateTransitionEvaluation {
                        from_state: current.to_string(),
                        to_state: transition.to_state.clone(),
                        duration_seconds,
                    },
                );
            } else if active_state.as_deref() != Some(transition.to_state.as_str()) {
                active_state = Some(transition.to_state.clone());
                transitioned = true;
            }
        }
    }

    let graph = active_state.as_deref().and_then(|state_name| {
        state_machine
            .states
            .iter()
            .find(|state| state.name == state_name)
            .map(|state| state.graph.clone())
    });

    AnimationStateMachineEvaluation {
        parameters: parameters.clone(),
        active_state,
        transitioned,
        graph,
        transition: transition_evaluation,
    }
}

fn condition_matches(
    parameters: &AnimationParameterMap,
    condition: &crate::asset::AnimationTransitionConditionAsset,
) -> bool {
    let Some(current) = parameters.get(&condition.parameter) else {
        return false;
    };
    if !animation_parameter_value_is_finite(current)
        || condition
            .value
            .as_ref()
            .is_some_and(|value| !animation_parameter_value_is_finite(value))
    {
        return false;
    }
    if matches!(
        condition.operator,
        AnimationConditionOperatorAsset::Triggered
    ) {
        return matches!(current, AnimationParameterValue::Trigger);
    }

    let Some(expected) = condition.value.as_ref() else {
        return false;
    };
    match condition.operator {
        AnimationConditionOperatorAsset::Triggered => unreachable!(),
        AnimationConditionOperatorAsset::Equal => current == expected,
        AnimationConditionOperatorAsset::NotEqual => current != expected,
        AnimationConditionOperatorAsset::Greater => {
            numeric_parameter(Some(current)) > numeric_parameter(Some(expected))
        }
        AnimationConditionOperatorAsset::GreaterEqual => {
            numeric_parameter(Some(current)) >= numeric_parameter(Some(expected))
        }
        AnimationConditionOperatorAsset::Less => {
            numeric_parameter(Some(current)) < numeric_parameter(Some(expected))
        }
        AnimationConditionOperatorAsset::LessEqual => {
            numeric_parameter(Some(current)) <= numeric_parameter(Some(expected))
        }
    }
}

fn state_machine_has_state(state_machine: &AnimationStateMachineAsset, name: &str) -> bool {
    state_machine.states.iter().any(|state| state.name == name)
}
