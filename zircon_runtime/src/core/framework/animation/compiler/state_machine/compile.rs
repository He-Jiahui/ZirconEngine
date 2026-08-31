//! Structural and semantic validation for state-machine authoring assets.

use std::collections::{BTreeMap, BTreeSet};

use robust::{Coord, orient2d};

use crate::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationParameterValue, AnimationStateAsset,
    AnimationStateKindAsset, AnimationStateMachineAsset, AnimationTransitionConditionAsset,
};

use super::model::{
    AnimationCompiledBlendSpace1DSample, AnimationCompiledBlendSpace2DSample,
    AnimationCompiledState, AnimationCompiledStateKind, AnimationCompiledStateMachine,
    AnimationCompiledStateMachineLayer, AnimationCompiledTransition,
    AnimationCompiledTransitionCondition, AnimationStateMachineCompilation,
};
use crate::core::framework::animation::compiler::{
    AnimationCompileDiagnostic, AnimationCompileElement, AnimationCompileSeverity,
    AnimationCompiledParameter, AnimationCompiledParameterKind, parameter_kind,
    parameter_value_is_finite,
};
use crate::core::math::{Real, Vec2};

const EMPTY_STATE_NAME: &str = "ZR-ANIM-COMP-STATE-001";
const DUPLICATE_STATE_NAME: &str = "ZR-ANIM-COMP-STATE-002";
const MISSING_ENTRY_STATE: &str = "ZR-ANIM-COMP-STATE-003";
const MISSING_TRANSITION_STATE: &str = "ZR-ANIM-COMP-STATE-004";
const INVALID_TRANSITION_DURATION: &str = "ZR-ANIM-COMP-STATE-005";
const INVALID_EXIT_TIME: &str = "ZR-ANIM-COMP-STATE-006";
const EMPTY_CONDITION_PARAMETER: &str = "ZR-ANIM-COMP-STATE-007";
const INVALID_TRIGGER_CONDITION: &str = "ZR-ANIM-COMP-STATE-008";
const INVALID_COMPARISON_CONDITION: &str = "ZR-ANIM-COMP-STATE-009";
const PARAMETER_TYPE_CONFLICT: &str = "ZR-ANIM-COMP-STATE-010";
const INVALID_LAYER_NAME: &str = "ZR-ANIM-COMP-STATE-011";
const INVALID_LAYER_VALUES: &str = "ZR-ANIM-COMP-STATE-012";
const INVALID_BLEND_PARAMETER: &str = "ZR-ANIM-COMP-STATE-013";
const INVALID_BLEND_SAMPLE: &str = "ZR-ANIM-COMP-STATE-014";
const DUPLICATE_BLEND_SAMPLE: &str = "ZR-ANIM-COMP-STATE-015";
const EMPTY_BLEND_SPACE: &str = "ZR-ANIM-COMP-STATE-016";
const COLLINEAR_BLEND_SPACE: &str = "ZR-ANIM-COMP-STATE-017";

/// Validates a state-machine asset and resolves its internal links to dense stable slots.
///
/// Referenced clips, graphs, and nested machines remain external dependencies; their loading and
/// cross-asset compilation are intentionally outside this pure source-only pass.
pub fn compile_animation_state_machine(
    asset: &AnimationStateMachineAsset,
) -> AnimationStateMachineCompilation {
    let mut diagnostics = Vec::new();
    let (source_states, state_slots) = collect_states(asset, &mut diagnostics);
    let entry_state = resolve_entry_state(asset, &state_slots, &mut diagnostics);
    let mut parameters = ParameterTable::default();
    let states = source_states
        .iter()
        .map(|state| {
            AnimationCompiledState::new(
                state.name.clone(),
                compile_state_kind(&state.kind, &state.name, &mut parameters, &mut diagnostics),
            )
        })
        .collect();
    let transitions = compile_transitions(asset, &state_slots, &mut parameters, &mut diagnostics);
    let layers = compile_layers(asset, &mut diagnostics);

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity() == AnimationCompileSeverity::Error)
    {
        return AnimationStateMachineCompilation::new(None, diagnostics);
    }

    AnimationStateMachineCompilation::new(
        Some(AnimationCompiledStateMachine::new(
            parameters.finish(),
            states,
            entry_state.expect("an error-free state machine has an entry state"),
            transitions,
            layers,
        )),
        diagnostics,
    )
}

fn collect_states<'a>(
    asset: &'a AnimationStateMachineAsset,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> (Vec<&'a AnimationStateAsset>, BTreeMap<String, usize>) {
    let mut states = Vec::with_capacity(asset.states.len());
    let mut slots = BTreeMap::new();
    for state in &asset.states {
        if state.name.trim().is_empty() {
            push_error(
                diagnostics,
                EMPTY_STATE_NAME,
                AnimationCompileElement::StateMachineState(state.name.clone()),
                "state name must not be empty",
            );
            continue;
        }
        if slots.contains_key(&state.name) {
            push_error(
                diagnostics,
                DUPLICATE_STATE_NAME,
                AnimationCompileElement::StateMachineState(state.name.clone()),
                "state names must be unique",
            );
            continue;
        }
        slots.insert(state.name.clone(), states.len());
        states.push(state);
    }
    (states, slots)
}

fn resolve_entry_state(
    asset: &AnimationStateMachineAsset,
    state_slots: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> Option<usize> {
    match state_slots.get(&asset.entry_state) {
        Some(slot) => Some(*slot),
        None => {
            push_error(
                diagnostics,
                MISSING_ENTRY_STATE,
                AnimationCompileElement::StateMachineState(asset.entry_state.clone()),
                "entry state must resolve to a declared state",
            );
            None
        }
    }
}

fn compile_state_kind(
    source: &AnimationStateKindAsset,
    state_name: &str,
    parameters: &mut ParameterTable,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> AnimationCompiledStateKind {
    let element = AnimationCompileElement::StateMachineState(state_name.to_string());
    match source {
        AnimationStateKindAsset::Clip { clip } => {
            AnimationCompiledStateKind::Clip { clip: clip.clone() }
        }
        AnimationStateKindAsset::GraphRef { graph } => AnimationCompiledStateKind::GraphRef {
            graph: graph.clone(),
        },
        AnimationStateKindAsset::SubMachine { state_machine } => {
            AnimationCompiledStateKind::SubMachine {
                state_machine: state_machine.clone(),
            }
        }
        AnimationStateKindAsset::BlendSpace1D(source) => {
            let parameter = parameters.register(
                &source.parameter,
                AnimationCompiledParameterKind::Scalar,
                element.clone(),
                diagnostics,
            );
            if source.samples.is_empty() {
                push_error(
                    diagnostics,
                    EMPTY_BLEND_SPACE,
                    element.clone(),
                    "blend-space 1D requires at least one sample",
                );
            }
            let mut positions = BTreeSet::new();
            let samples = source
                .samples
                .iter()
                .map(|sample| {
                    if !sample.position.is_finite() {
                        push_error(
                            diagnostics,
                            INVALID_BLEND_SAMPLE,
                            element.clone(),
                            "blend-space 1D sample positions must be finite",
                        );
                    } else if !positions.insert(canonical_real_bits(sample.position)) {
                        push_error(
                            diagnostics,
                            DUPLICATE_BLEND_SAMPLE,
                            element.clone(),
                            "blend-space 1D sample positions must be unique",
                        );
                    }
                    AnimationCompiledBlendSpace1DSample {
                        position: sample.position,
                        graph: sample.graph.clone(),
                    }
                })
                .collect();
            AnimationCompiledStateKind::BlendSpace1D { parameter, samples }
        }
        AnimationStateKindAsset::BlendSpace2D(source) => {
            let parameter = parameters.register(
                &source.parameter,
                AnimationCompiledParameterKind::Vec2,
                element.clone(),
                diagnostics,
            );
            if source.samples.len() < 3 {
                push_error(
                    diagnostics,
                    EMPTY_BLEND_SPACE,
                    element.clone(),
                    "blend-space 2D requires at least three samples",
                );
            }
            let mut position_keys = BTreeSet::new();
            let mut positions = Vec::with_capacity(source.samples.len());
            let samples = source
                .samples
                .iter()
                .map(|sample| {
                    if !sample
                        .position
                        .to_array()
                        .iter()
                        .all(|value| value.is_finite())
                    {
                        push_error(
                            diagnostics,
                            INVALID_BLEND_SAMPLE,
                            element.clone(),
                            "blend-space 2D sample positions must be finite",
                        );
                    } else if !position_keys.insert(canonical_point_bits(sample.position)) {
                        push_error(
                            diagnostics,
                            DUPLICATE_BLEND_SAMPLE,
                            element.clone(),
                            "blend-space 2D sample positions must be unique",
                        );
                    } else {
                        positions.push(sample.position);
                    }
                    AnimationCompiledBlendSpace2DSample {
                        position: sample.position,
                        graph: sample.graph.clone(),
                    }
                })
                .collect();
            if positions.len() >= 3 && !contains_non_collinear_points(&positions) {
                push_error(
                    diagnostics,
                    COLLINEAR_BLEND_SPACE,
                    element,
                    "blend-space 2D sample positions must not be collinear",
                );
            }
            AnimationCompiledStateKind::BlendSpace2D { parameter, samples }
        }
    }
}

fn contains_non_collinear_points(points: &[Vec2]) -> bool {
    let Some(first) = points.first().copied() else {
        return false;
    };
    let Some(second) = points.iter().copied().skip(1).find(|point| *point != first) else {
        return false;
    };
    points
        .iter()
        .copied()
        .any(|third| orient2d(coord(first), coord(second), coord(third)) != 0.0)
}

fn canonical_point_bits(point: Vec2) -> [u32; 2] {
    point.to_array().map(canonical_real_bits)
}

fn canonical_real_bits(value: Real) -> u32 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

fn coord(point: Vec2) -> Coord<Real> {
    let [x, y] = point.to_array();
    Coord { x, y }
}

fn compile_transitions(
    asset: &AnimationStateMachineAsset,
    state_slots: &BTreeMap<String, usize>,
    parameters: &mut ParameterTable,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> Vec<AnimationCompiledTransition> {
    asset
        .transitions
        .iter()
        .enumerate()
        .map(|(transition_index, transition)| {
            let element = transition_element(
                transition_index,
                &transition.from_state,
                &transition.to_state,
            );
            let from_state = resolve_transition_state(
                transition_index,
                "from",
                &transition.from_state,
                &transition.from_state,
                &transition.to_state,
                state_slots,
                diagnostics,
            );
            let to_state = resolve_transition_state(
                transition_index,
                "to",
                &transition.to_state,
                &transition.from_state,
                &transition.to_state,
                state_slots,
                diagnostics,
            );
            if !transition.duration_seconds.is_finite() || transition.duration_seconds < 0.0 {
                push_error(
                    diagnostics,
                    INVALID_TRANSITION_DURATION,
                    element.clone(),
                    "transition duration must be finite and non-negative",
                );
            }
            if transition
                .exit_time
                .is_some_and(|exit_time| !exit_time.is_finite() || exit_time < 0.0)
            {
                push_error(
                    diagnostics,
                    INVALID_EXIT_TIME,
                    element.clone(),
                    "transition exit time must be finite and non-negative when present",
                );
            }
            let conditions = transition
                .conditions
                .iter()
                .enumerate()
                .map(|(condition_index, condition)| {
                    compile_transition_condition(
                        transition_index,
                        condition_index,
                        condition,
                        parameters,
                        diagnostics,
                    )
                })
                .collect();
            AnimationCompiledTransition::new(
                from_state,
                to_state,
                transition.duration_seconds,
                transition.exit_time,
                transition.interruption,
                conditions,
            )
        })
        .collect()
}

fn resolve_transition_state(
    transition_index: usize,
    endpoint: &str,
    name: &str,
    from_state: &str,
    to_state: &str,
    state_slots: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> usize {
    match state_slots.get(name) {
        Some(slot) => *slot,
        None => {
            push_error(
                diagnostics,
                MISSING_TRANSITION_STATE,
                transition_element(transition_index, from_state, to_state),
                format!(
                    "transition {endpoint}_state `{name}` does not resolve to a declared state"
                ),
            );
            usize::MAX
        }
    }
}

fn compile_transition_condition(
    transition_index: usize,
    condition_index: usize,
    condition: &AnimationTransitionConditionAsset,
    parameters: &mut ParameterTable,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> AnimationCompiledTransitionCondition {
    let element = AnimationCompileElement::StateMachineCondition {
        transition_index,
        condition_index,
        parameter: condition.parameter.clone(),
    };
    let expected_kind = match condition.operator {
        AnimationConditionOperatorAsset::Triggered => {
            if condition.value.is_some() {
                push_error(
                    diagnostics,
                    INVALID_TRIGGER_CONDITION,
                    element.clone(),
                    "triggered conditions must not carry a comparison value",
                );
            }
            AnimationCompiledParameterKind::Trigger
        }
        AnimationConditionOperatorAsset::Greater
        | AnimationConditionOperatorAsset::GreaterEqual
        | AnimationConditionOperatorAsset::Less
        | AnimationConditionOperatorAsset::LessEqual => {
            let Some(value) = condition.value.as_ref() else {
                push_error(
                    diagnostics,
                    INVALID_COMPARISON_CONDITION,
                    element.clone(),
                    "numeric transition conditions require a comparison value",
                );
                return invalid_compiled_condition(condition);
            };
            if !matches!(
                value,
                AnimationParameterValue::Integer(_) | AnimationParameterValue::Scalar(_)
            ) || !parameter_value_is_finite(value)
            {
                push_error(
                    diagnostics,
                    INVALID_COMPARISON_CONDITION,
                    element.clone(),
                    "numeric transition conditions require a finite integer or scalar value",
                );
            }
            AnimationCompiledParameterKind::Numeric
        }
        AnimationConditionOperatorAsset::Equal | AnimationConditionOperatorAsset::NotEqual => {
            let Some(value) = condition.value.as_ref() else {
                push_error(
                    diagnostics,
                    INVALID_COMPARISON_CONDITION,
                    element.clone(),
                    "equality transition conditions require a comparison value",
                );
                return invalid_compiled_condition(condition);
            };
            if matches!(value, AnimationParameterValue::Trigger)
                || !parameter_value_is_finite(value)
            {
                push_error(
                    diagnostics,
                    INVALID_COMPARISON_CONDITION,
                    element.clone(),
                    "equality transition conditions require a finite non-trigger value",
                );
            }
            parameter_kind(value)
        }
    };
    let parameter = parameters.register(&condition.parameter, expected_kind, element, diagnostics);
    AnimationCompiledTransitionCondition::new(
        parameter,
        condition.operator,
        condition.value.clone(),
    )
}

fn invalid_compiled_condition(
    condition: &AnimationTransitionConditionAsset,
) -> AnimationCompiledTransitionCondition {
    AnimationCompiledTransitionCondition::new(
        usize::MAX,
        condition.operator,
        condition.value.clone(),
    )
}

fn compile_layers(
    asset: &AnimationStateMachineAsset,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> Vec<AnimationCompiledStateMachineLayer> {
    let mut names = BTreeSet::new();
    asset
        .layers
        .iter()
        .map(|layer| {
            let element = AnimationCompileElement::StateMachineLayer(layer.name.clone());
            if layer.name.trim().is_empty() || !names.insert(layer.name.as_str()) {
                push_error(
                    diagnostics,
                    INVALID_LAYER_NAME,
                    element.clone(),
                    "layer names must be non-empty and unique",
                );
            }
            if !layer.weight.is_finite()
                || !(0.0..=1.0).contains(&layer.weight)
                || layer
                    .mask_weights
                    .iter()
                    .any(|weight| !weight.is_finite() || !(0.0..=1.0).contains(weight))
            {
                push_error(
                    diagnostics,
                    INVALID_LAYER_VALUES,
                    element,
                    "layer and mask weights must be finite values in the inclusive 0..=1 range",
                );
            }
            AnimationCompiledStateMachineLayer::new(
                layer.name.clone(),
                layer.state_machine.clone(),
                layer.weight,
                layer.blend_mode,
                layer.mask_weights.clone(),
            )
        })
        .collect()
}

fn transition_element(
    transition_index: usize,
    from_state: &str,
    to_state: &str,
) -> AnimationCompileElement {
    AnimationCompileElement::StateMachineTransition {
        transition_index,
        from_state: from_state.to_string(),
        to_state: to_state.to_string(),
    }
}

fn push_error(
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    code: &'static str,
    element: AnimationCompileElement,
    message: impl Into<String>,
) {
    diagnostics.push(AnimationCompileDiagnostic::new(
        code,
        AnimationCompileSeverity::Error,
        element,
        message,
    ));
}

#[derive(Default)]
struct ParameterTable {
    indexes: BTreeMap<String, usize>,
    parameters: Vec<AnimationCompiledParameter>,
}

impl ParameterTable {
    fn register(
        &mut self,
        name: &str,
        requested_kind: AnimationCompiledParameterKind,
        element: AnimationCompileElement,
        diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    ) -> usize {
        if name.trim().is_empty() {
            push_error(
                diagnostics,
                if matches!(&element, AnimationCompileElement::StateMachineState(_)) {
                    INVALID_BLEND_PARAMETER
                } else {
                    EMPTY_CONDITION_PARAMETER
                },
                element,
                "state-machine parameter name must not be empty",
            );
            return usize::MAX;
        }
        if let Some(index) = self.indexes.get(name).copied() {
            let existing_kind = self.parameters[index].kind();
            if let Some(merged_kind) = merge_parameter_kinds(existing_kind, requested_kind) {
                self.parameters[index].set_kind(merged_kind);
            } else {
                push_error(
                    diagnostics,
                    PARAMETER_TYPE_CONFLICT,
                    element,
                    format!(
                        "parameter `{name}` is required as both {existing_kind:?} and {requested_kind:?}"
                    ),
                );
            }
            return index;
        }
        let index = self.parameters.len();
        self.indexes.insert(name.to_string(), index);
        self.parameters.push(AnimationCompiledParameter::declared(
            name.to_string(),
            requested_kind,
        ));
        index
    }

    fn finish(self) -> Vec<AnimationCompiledParameter> {
        self.parameters
    }
}

fn merge_parameter_kinds(
    existing: AnimationCompiledParameterKind,
    requested: AnimationCompiledParameterKind,
) -> Option<AnimationCompiledParameterKind> {
    use AnimationCompiledParameterKind::{Integer, Numeric, Scalar};

    if existing == requested {
        return Some(existing);
    }
    match (existing, requested) {
        (Numeric, Integer | Scalar) => Some(requested),
        (Integer | Scalar, Numeric) => Some(existing),
        (Integer, Scalar) | (Scalar, Integer) => Some(Numeric),
        _ => None,
    }
}

#[cfg(test)]
mod optimization_batch_20260830co_runtime_tests {
    const SYNTHETIC_STATE_COUNT: usize = 32_768;

    #[test]
    fn optimization_batch_20260830co_runtime_state_collection_reserves_authored_upper_bound() {
        let source = include_str!("compile.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("state machine compiler implementation");

        assert!(implementation.contains("Vec::with_capacity(asset.states.len())"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830co_runtime_state_collection_capacity_evidence() {
        let legacy_growth_events = collect_growth_events(false);
        let optimized_growth_events = collect_growth_events(true);

        println!(
            "RUNTIME502_STATE_MACHINE_STATE_CAPACITY_BENCH_V1 states={SYNTHETIC_STATE_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
        );
        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
    }

    fn collect_growth_events(reserve_upper_bound: bool) -> usize {
        let capacity = usize::from(reserve_upper_bound) * SYNTHETIC_STATE_COUNT;
        let mut states = Vec::with_capacity(capacity);
        let mut growth_events = 0;
        for state in 0..SYNTHETIC_STATE_COUNT {
            let previous_capacity = states.capacity();
            states.push(state);
            growth_events += usize::from(states.capacity() != previous_capacity);
        }
        std::hint::black_box(states);
        growth_events
    }
}
