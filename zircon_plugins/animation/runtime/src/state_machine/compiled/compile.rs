use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::core::framework::animation::compiler::state_machine::{
    compile_animation_state_machine, AnimationCompiledStateKind, AnimationCompiledStateMachine,
    AnimationCompiledTransitionCondition,
};
use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationStateMachineAsset,
};

use crate::state_machine::blend_space::{BlendSpace1D, BlendSpace2D};
use crate::state_machine::condition_expression::{compile_shared_conditions, ParameterSlot};
use crate::state_machine::layer::CompiledStateMachineLayers;

use super::{
    AnimationStateMachineCompileError, CompiledAnimationStateMachine, CompiledState,
    CompiledStateKind, CompiledTransition, StateSlot,
};

/// Compiles source semantics once in the framework and lowers the accepted IR for evaluation.
pub fn compile_animation_state_machine_runtime(
    source: &AnimationStateMachineAsset,
) -> Result<CompiledAnimationStateMachine, AnimationStateMachineCompileError> {
    compile_animation_state_machine_runtime_bundle(source).map(|(machine, _)| machine)
}

pub(crate) fn compile_animation_state_machine_runtime_bundle(
    source: &AnimationStateMachineAsset,
) -> Result<
    (CompiledAnimationStateMachine, CompiledStateMachineLayers),
    AnimationStateMachineCompileError,
> {
    let compilation = compile_animation_state_machine(source);
    let Some(artifact) = compilation.artifact() else {
        return Err(AnimationStateMachineCompileError::SourceDiagnostics(
            compilation.diagnostics().to_vec(),
        ));
    };
    Ok((
        CompiledAnimationStateMachine::from_compiled(artifact)?,
        CompiledStateMachineLayers::from_compiled(artifact.layers()),
    ))
}

impl CompiledAnimationStateMachine {
    pub(crate) fn from_compiled(
        artifact: &AnimationCompiledStateMachine,
    ) -> Result<Self, AnimationStateMachineCompileError> {
        let mut state_slots = BTreeMap::new();
        let states = artifact
            .states()
            .iter()
            .enumerate()
            .map(|(index, state)| {
                let slot = StateSlot::new(index)
                    .ok_or(AnimationStateMachineCompileError::CapacityExceeded)?;
                state_slots.insert(state.name().to_string(), slot);
                Ok(CompiledState {
                    name: state.name().to_string(),
                    kind: compile_state_kind(state.kind())?,
                })
            })
            .collect::<Result<Vec<_>, AnimationStateMachineCompileError>>()?;
        let parameter_names = artifact
            .parameters()
            .iter()
            .map(|parameter| parameter.name().to_string())
            .collect::<Vec<_>>();
        let entry = StateSlot::new(artifact.entry_state())
            .ok_or(AnimationStateMachineCompileError::CapacityExceeded)?;
        let mut transitions = vec![Vec::new(); states.len()];
        for transition in artifact.transitions() {
            let from = StateSlot::new(transition.from_state())
                .ok_or(AnimationStateMachineCompileError::CapacityExceeded)?;
            let to = StateSlot::new(transition.to_state())
                .ok_or(AnimationStateMachineCompileError::CapacityExceeded)?;
            transitions[from.index()].push(CompiledTransition {
                to,
                desc: crate::TransitionDesc::new(transition.duration_seconds())
                    .with_optional_exit_time(transition.exit_time())
                    .with_interruption(transition.interruption().into()),
                conditions: compile_shared_conditions(transition.conditions())?,
                consumed_triggers: compile_consumed_triggers(
                    transition.conditions(),
                    &parameter_names,
                ),
            });
        }
        Ok(Self {
            states: states.into_boxed_slice(),
            state_slots,
            parameter_names: Arc::from(parameter_names),
            entry,
            transitions: transitions.into_iter().map(Vec::into_boxed_slice).collect(),
        })
    }
}

fn compile_state_kind(
    source: &AnimationCompiledStateKind,
) -> Result<CompiledStateKind, AnimationStateMachineCompileError> {
    match source {
        AnimationCompiledStateKind::GraphRef { graph } => {
            Ok(CompiledStateKind::GraphRef(graph.clone()))
        }
        AnimationCompiledStateKind::BlendSpace1D { parameter, samples } => {
            let parameter = ParameterSlot::new(*parameter)?;
            let graphs = samples
                .iter()
                .map(|sample| sample.graph.clone())
                .collect::<Vec<_>>();
            Ok(CompiledStateKind::BlendSpace1D {
                parameter,
                blend: BlendSpace1D::from_compiled(samples)?,
                graphs: graphs.into_boxed_slice(),
            })
        }
        AnimationCompiledStateKind::BlendSpace2D { parameter, samples } => {
            let parameter = ParameterSlot::new(*parameter)?;
            let graphs = samples
                .iter()
                .map(|sample| sample.graph.clone())
                .collect::<Vec<_>>();
            Ok(CompiledStateKind::BlendSpace2D {
                parameter,
                blend: BlendSpace2D::from_compiled(samples)?,
                graphs: graphs.into_boxed_slice(),
            })
        }
        AnimationCompiledStateKind::Clip { clip } => Ok(CompiledStateKind::Clip(clip.clone())),
        AnimationCompiledStateKind::SubMachine { state_machine } => {
            Ok(CompiledStateKind::SubMachine(state_machine.clone()))
        }
    }
}

fn compile_consumed_triggers(
    conditions: &[AnimationCompiledTransitionCondition],
    parameter_names: &[String],
) -> Arc<[String]> {
    let mut triggers = Vec::new();
    for condition in conditions {
        if condition.operator() == AnimationConditionOperatorAsset::Triggered {
            let name = &parameter_names[condition.parameter()];
            if !triggers.contains(name) {
                triggers.push(name.clone());
            }
        }
    }
    Arc::from(triggers)
}
