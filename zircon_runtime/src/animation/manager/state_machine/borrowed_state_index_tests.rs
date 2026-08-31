use std::{hint::black_box, time::Instant};

use super::*;
use crate::core::framework::animation::{
    AnimationStateAsset, AnimationStateKindAsset, AnimationStateTransitionAsset,
};
use crate::core::resource::{AssetReference, ResourceLocator};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime08c_batch_state_machine_index_preserves_first_state_and_transition_order() {
    let first_target_graph = asset_reference("res://animation/first-target.graph.zranim");
    let machine = AnimationStateMachineAsset {
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

    let evaluation = evaluate_state_machine(&machine, Some("idle"), &AnimationParameterMap::new());

    assert_eq!(evaluation.active_state.as_deref(), Some("target"));
    assert!(evaluation.transitioned);
    assert_eq!(evaluation.graph, Some(first_target_graph));
}

#[test]
fn runtime08c_batch_state_machine_uses_borrowed_state_index() {
    let source = include_str!("../state_machine.rs");
    let evaluation = bounded_source(
        source,
        "pub(super) fn evaluate_state_machine(",
        "fn condition_matches(",
    );

    assert!(evaluation.contains("HashMap::with_capacity"));
    assert!(evaluation.contains("states_by_name.entry"));
    assert!(evaluation.contains(".or_insert(state)"));
    assert!(evaluation.contains("STATE_INDEX_MIN_PROJECTED_COMPARISONS"));
    assert!(evaluation.contains("let state_by_name"));
    assert!(evaluation.contains("states_by_name.get(name).copied()"));
    assert!(!evaluation.contains("state_machine_has_state"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime08c_batch_state_machine_borrowed_state_index_p95() {
    const STATE_COUNT: usize = 2_048;
    const INVALID_TRANSITIONS: usize = 256;
    const EVALUATIONS: usize = 2;
    let machine = benchmark_machine(STATE_COUNT, INVALID_TRANSITIONS);
    let parameters = AnimationParameterMap::new();
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(EVALUATIONS, || {
                legacy_evaluate_state_machine(black_box(&machine), black_box(&parameters))
            }));
            optimized_ns.push(measure_ns(EVALUATIONS, || {
                evaluate_state_machine(
                    black_box(&machine),
                    Some("state-0000"),
                    black_box(&parameters),
                )
                .active_state
                .map(|state| state.len())
                .unwrap_or_default()
            }));
        } else {
            optimized_ns.push(measure_ns(EVALUATIONS, || {
                evaluate_state_machine(
                    black_box(&machine),
                    Some("state-0000"),
                    black_box(&parameters),
                )
                .active_state
                .map(|state| state.len())
                .unwrap_or_default()
            }));
            legacy_ns.push(measure_ns(EVALUATIONS, || {
                legacy_evaluate_state_machine(black_box(&machine), black_box(&parameters))
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "borrowed state index P95 must be at least 90% below repeated state scans: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME08C_STATE_MACHINE_BORROWED_STATE_INDEX_BENCH_V1 states={STATE_COUNT} invalid_transitions={INVALID_TRANSITIONS} evaluations_per_sample={EVALUATIONS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_state_comparisons_per_sample={} optimized_state_index_visits_per_sample={} optimized_hash_lookups_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        (1 + STATE_COUNT * (INVALID_TRANSITIONS + 2)) * EVALUATIONS,
        STATE_COUNT * EVALUATIONS,
        (INVALID_TRANSITIONS + 3) * EVALUATIONS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn benchmark_machine(state_count: usize, invalid_transitions: usize) -> AnimationStateMachineAsset {
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

fn legacy_evaluate_state_machine(
    machine: &AnimationStateMachineAsset,
    parameters: &AnimationParameterMap,
) -> usize {
    let mut active_state = Some("state-0000")
        .filter(|state| legacy_state_machine_has_state(machine, state))
        .map(ToOwned::to_owned)
        .or_else(|| {
            legacy_state_machine_has_state(machine, &machine.entry_state)
                .then(|| machine.entry_state.clone())
        });
    let mut transitioned = false;
    let mut transition_evaluation = None;

    if let Some(current) = active_state.as_deref() {
        if let Some(transition) = machine.transitions.iter().find(|transition| {
            transition.from_state == current
                && legacy_state_machine_has_state(machine, &transition.to_state)
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
        machine
            .states
            .iter()
            .find(|state| state.name == state_name)
            .and_then(|state| state.kind.graph_reference().cloned())
    });
    let evaluation = AnimationStateMachineEvaluation {
        parameters: parameters.clone(),
        active_state,
        transitioned,
        graph,
        transition: transition_evaluation,
    };
    evaluation
        .active_state
        .map(|state| state.len())
        .unwrap_or_default()
        + usize::from(evaluation.transitioned)
        + usize::from(evaluation.graph.is_some())
}

fn legacy_state_machine_has_state(machine: &AnimationStateMachineAsset, name: &str) -> bool {
    machine.states.iter().any(|state| state.name == name)
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

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
