use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationStateKindAsset, AnimationStateMachineAsset,
    AnimationTransitionConditionAsset,
};

use crate::state_machine::condition_expression::{ParameterTableBuilder, compile_all_conditions};
use crate::{BlendSpace1D, BlendSpace2D, BlendSpacePoint1D, BlendSpacePoint2D};

use super::{
    AnimationStateMachineCompileError, CompiledAnimationStateMachine, CompiledState,
    CompiledStateKind, CompiledTransition, StateSlot,
};

impl CompiledAnimationStateMachine {
    pub fn compile(
        source: &AnimationStateMachineAsset,
    ) -> Result<Self, AnimationStateMachineCompileError> {
        let mut parameters = ParameterTableBuilder::default();
        let (states, state_slots) = compile_states(source, &mut parameters)?;
        let entry = resolve_state(&state_slots, &source.entry_state)?;
        let mut transitions = vec![Vec::new(); states.len()];
        for transition in &source.transitions {
            let from = resolve_state(&state_slots, &transition.from_state)?;
            let to = resolve_state(&state_slots, &transition.to_state)?;
            transitions[from.index()].push(CompiledTransition {
                to,
                desc: crate::TransitionDesc::new(transition.duration_seconds)
                    .with_optional_exit_time(transition.exit_time)
                    .with_interruption(transition.interruption.into()),
                conditions: compile_all_conditions(&transition.conditions, &mut parameters)?,
                consumed_triggers: compile_consumed_triggers(&transition.conditions),
            });
        }
        Ok(Self {
            states: states.into_boxed_slice(),
            state_slots,
            parameter_names: parameters.finish(),
            entry,
            transitions: transitions.into_iter().map(Vec::into_boxed_slice).collect(),
        })
    }
}

fn compile_consumed_triggers(conditions: &[AnimationTransitionConditionAsset]) -> Arc<[String]> {
    let mut triggers = Vec::new();
    for condition in conditions {
        if condition.operator == AnimationConditionOperatorAsset::Triggered
            && !triggers.contains(&condition.parameter)
        {
            triggers.push(condition.parameter.clone());
        }
    }
    Arc::from(triggers)
}

fn compile_states(
    source: &AnimationStateMachineAsset,
    parameters: &mut ParameterTableBuilder,
) -> Result<(Vec<CompiledState>, BTreeMap<String, StateSlot>), AnimationStateMachineCompileError> {
    let mut states = Vec::with_capacity(source.states.len());
    let mut slots = BTreeMap::new();
    for state in &source.states {
        let slot = StateSlot::new(states.len())
            .ok_or(AnimationStateMachineCompileError::CapacityExceeded)?;
        if slots.insert(state.name.clone(), slot).is_some() {
            return Err(AnimationStateMachineCompileError::DuplicateState {
                name: state.name.clone(),
            });
        }
        states.push(CompiledState {
            name: state.name.clone(),
            kind: compile_state_kind(&state.kind, parameters)?,
        });
    }
    Ok((states, slots))
}

fn compile_state_kind(
    source: &AnimationStateKindAsset,
    parameters: &mut ParameterTableBuilder,
) -> Result<CompiledStateKind, AnimationStateMachineCompileError> {
    match source {
        AnimationStateKindAsset::GraphRef { graph } => {
            Ok(CompiledStateKind::GraphRef(graph.clone()))
        }
        AnimationStateKindAsset::BlendSpace1D(source) => {
            let parameter = parameters.intern(&source.parameter)?;
            let graphs = source
                .samples
                .iter()
                .map(|sample| sample.graph.clone())
                .collect::<Vec<_>>();
            let points = source
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| {
                    Ok(BlendSpacePoint1D::new(
                        sample.position,
                        u32::try_from(index)
                            .map_err(|_| AnimationStateMachineCompileError::CapacityExceeded)?,
                    ))
                })
                .collect::<Result<Vec<_>, AnimationStateMachineCompileError>>()?;
            Ok(CompiledStateKind::BlendSpace1D {
                parameter,
                blend: BlendSpace1D::compile(points)?,
                graphs: graphs.into_boxed_slice(),
            })
        }
        AnimationStateKindAsset::BlendSpace2D(source) => {
            let parameter = parameters.intern(&source.parameter)?;
            let graphs = source
                .samples
                .iter()
                .map(|sample| sample.graph.clone())
                .collect::<Vec<_>>();
            let points = source
                .samples
                .iter()
                .enumerate()
                .map(|(index, sample)| {
                    Ok(BlendSpacePoint2D::new(
                        sample.position,
                        u32::try_from(index)
                            .map_err(|_| AnimationStateMachineCompileError::CapacityExceeded)?,
                    ))
                })
                .collect::<Result<Vec<_>, AnimationStateMachineCompileError>>()?;
            Ok(CompiledStateKind::BlendSpace2D {
                parameter,
                blend: BlendSpace2D::compile(points)?,
                graphs: graphs.into_boxed_slice(),
            })
        }
        AnimationStateKindAsset::Clip { clip } => Ok(CompiledStateKind::Clip(clip.clone())),
        AnimationStateKindAsset::SubMachine { state_machine } => {
            Ok(CompiledStateKind::SubMachine(state_machine.clone()))
        }
    }
}

fn resolve_state(
    states: &BTreeMap<String, StateSlot>,
    name: &str,
) -> Result<StateSlot, AnimationStateMachineCompileError> {
    states
        .get(name)
        .copied()
        .ok_or_else(|| AnimationStateMachineCompileError::MissingState { name: name.into() })
}
