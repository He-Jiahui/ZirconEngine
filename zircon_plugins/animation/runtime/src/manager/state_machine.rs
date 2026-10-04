//! 管理器同步状态机评估保留作者状态和转换声明顺序；帧管线使用编译后的实例状态。
use std::collections::HashMap;

use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationStateMachineAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationParameterMap, AnimationParameterValue, AnimationStateMachineEvaluation,
};
use zircon_runtime::core::math::Real;

use super::parameters::numeric_parameter;
use super::sampling::animation_parameter_value_is_finite;

const STATE_INDEX_MIN_PROJECTED_COMPARISONS: usize = 2_048;

fn should_index_states(state_count: usize, transition_count: usize) -> bool {
    state_count.saturating_mul(transition_count.saturating_add(2))
        >= STATE_INDEX_MIN_PROJECTED_COMPARISONS
}

/// 同步评估当前作者状态；正持续时间返回转换请求，实例进度与交叉淡入须由上层保存。
pub(super) fn evaluate_state_machine(
    state_machine: &AnimationStateMachineAsset,
    current_state: Option<&str>,
    parameters: &AnimationParameterMap,
) -> AnimationStateMachineEvaluation {
    let states_by_name =
        should_index_states(state_machine.states.len(), state_machine.transitions.len()).then(
            || {
                let mut states_by_name = HashMap::with_capacity(state_machine.states.len());
                for state in &state_machine.states {
                    states_by_name.entry(state.name.as_str()).or_insert(state);
                }
                states_by_name
            },
        );
    let state_by_name = |name: &str| match states_by_name.as_ref() {
        Some(states_by_name) => states_by_name.get(name).copied(),
        None => state_machine.states.iter().find(|state| state.name == name),
    };
    let mut active_state = current_state
        .filter(|state| state_by_name(state).is_some())
        .map(ToOwned::to_owned)
        .or_else(|| {
            state_by_name(state_machine.entry_state.as_str())
                .is_some()
                .then(|| state_machine.entry_state.clone())
        });
    let mut transitioned = false;
    let mut transition_evaluation = None;

    if let Some(current) = active_state.as_deref() {
        if let Some(transition) = state_machine.transitions.iter().find(|transition| {
            transition.from_state == current
                && state_by_name(transition.to_state.as_str()).is_some()
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
                    zircon_runtime::core::framework::animation::AnimationStateTransitionEvaluation {
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

    let graph = active_state
        .as_deref()
        .and_then(state_by_name)
        .and_then(|state| state.kind.graph_reference().cloned());

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
    condition: &zircon_runtime::core::framework::animation::AnimationTransitionConditionAsset,
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

#[cfg(test)]
#[path = "tests/state_machine_optimization_batch_20260830cl_tests.rs"]
mod optimization_batch_20260830cl_tests;
