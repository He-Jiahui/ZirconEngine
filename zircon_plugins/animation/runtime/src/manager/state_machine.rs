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
mod optimization_batch_20260830cl_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::animation::{
        AnimationParameterMap, AnimationStateAsset, AnimationStateKindAsset,
        AnimationStateMachineAsset, AnimationStateTransitionAsset,
    };
    use zircon_runtime::core::resource::{AssetReference, ResourceLocator};

    use super::{condition_matches, evaluate_state_machine, should_index_states};

    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_20260830cl_state_index_preserves_first_state_and_transition_order() {
        let first_target_graph = asset_reference("res://animation/first-target.graph.zranim");
        let mut machine = AnimationStateMachineAsset {
            name: None,
            entry_state: "idle".to_string(),
            states: vec![
                state("idle", "res://animation/idle.graph.zranim"),
                AnimationStateAsset::graph_ref("target", first_target_graph.clone()),
                state("target", "res://animation/duplicate-target.graph.zranim"),
            ],
            transitions: vec![transition("idle", "missing"), transition("idle", "target")],
            layers: Vec::new(),
        };
        machine.states.extend((0..64).map(|index| {
            state(
                &format!("padding-{index:02}"),
                &format!("res://animation/padding-{index:02}.graph.zranim"),
            )
        }));
        machine
            .transitions
            .extend((0..30).map(|index| transition("idle", &format!("missing-{index:02}"))));

        assert!(should_index_states(
            machine.states.len(),
            machine.transitions.len()
        ));
        let evaluation = evaluate_state_machine(
            &machine,
            Some("missing-current"),
            &AnimationParameterMap::new(),
        );

        assert_eq!(evaluation.active_state.as_deref(), Some("target"));
        assert!(evaluation.transitioned);
        assert_eq!(evaluation.graph, Some(first_target_graph));
    }

    #[test]
    fn optimization_batch_20260830cl_state_index_is_adaptive() {
        assert!(!should_index_states(8, 16));
        assert!(!should_index_states(16, 32));
        assert!(should_index_states(32, 64));
        assert!(should_index_states(128, 256));
    }

    #[test]
    #[ignore = "release-only adaptive state lookup benchmark"]
    fn optimization_batch_20260830cl_state_index_release_benchmark() {
        const STATE_COUNT: usize = 1_024;
        const INVALID_TRANSITIONS: usize = 2_048;
        let machine = benchmark_machine(STATE_COUNT, INVALID_TRANSITIONS);
        let parameters = AnimationParameterMap::new();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_ns(|| legacy_evaluate(&machine, &parameters)));
                optimized_samples.push(measure_ns(|| {
                    evaluate_state_machine(&machine, Some("state-0000"), &parameters)
                        .active_state
                        .map(|state| state.len())
                        .unwrap_or_default()
                }));
            } else {
                optimized_samples.push(measure_ns(|| {
                    evaluate_state_machine(&machine, Some("state-0000"), &parameters)
                        .active_state
                        .map(|state| state.len())
                        .unwrap_or_default()
                }));
                legacy_samples.push(measure_ns(|| legacy_evaluate(&machine, &parameters)));
            }
        }

        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME170_PLUGIN_STATE_ADAPTIVE_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} state_count={STATE_COUNT} invalid_transitions={INVALID_TRANSITIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
            "adaptive state lookup must reduce large transition-set P95 by at least 80%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn benchmark_machine(
        state_count: usize,
        invalid_transitions: usize,
    ) -> AnimationStateMachineAsset {
        let states = (0..state_count)
            .map(|index| {
                state(
                    &format!("state-{index:04}"),
                    &format!("res://animation/state-{index:04}.graph.zranim"),
                )
            })
            .collect();
        let mut transitions = (0..invalid_transitions)
            .map(|index| transition("state-0000", &format!("missing-{index:04}")))
            .collect::<Vec<_>>();
        transitions.push(transition(
            "state-0000",
            &format!("state-{:04}", state_count - 1),
        ));
        AnimationStateMachineAsset {
            name: None,
            entry_state: "state-0000".to_string(),
            states,
            transitions,
            layers: Vec::new(),
        }
    }

    fn legacy_evaluate(
        machine: &AnimationStateMachineAsset,
        parameters: &AnimationParameterMap,
    ) -> usize {
        let mut active_state = Some("state-0000")
            .filter(|name| machine.states.iter().any(|state| state.name == *name))
            .map(ToOwned::to_owned);
        if let Some(current) = active_state.as_deref() {
            if let Some(transition) = machine.transitions.iter().find(|transition| {
                transition.from_state == current
                    && machine
                        .states
                        .iter()
                        .any(|state| state.name == transition.to_state)
                    && transition
                        .conditions
                        .iter()
                        .all(|condition| condition_matches(parameters, condition))
            }) {
                active_state = Some(transition.to_state.clone());
            }
        }
        let graph = active_state.as_deref().and_then(|name| {
            machine
                .states
                .iter()
                .find(|state| state.name == name)
                .and_then(|state| state.kind.graph_reference())
        });
        active_state.map(|state| state.len()).unwrap_or_default() + usize::from(graph.is_some())
    }

    fn state(name: &str, graph: &str) -> AnimationStateAsset {
        AnimationStateAsset {
            name: name.to_string(),
            kind: AnimationStateKindAsset::GraphRef {
                graph: asset_reference(graph),
            },
        }
    }

    fn transition(from_state: &str, to_state: &str) -> AnimationStateTransitionAsset {
        AnimationStateTransitionAsset {
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            duration_seconds: 0.0,
            exit_time: None,
            interruption: Default::default(),
            conditions: Vec::new(),
        }
    }

    fn asset_reference(locator: &str) -> AssetReference {
        AssetReference::from_locator(ResourceLocator::parse(locator).expect("asset locator"))
    }

    fn measure_ns(mut operation: impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        black_box(operation());
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
