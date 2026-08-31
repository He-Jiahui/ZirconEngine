use std::sync::Arc;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::scene::AnimationStateTransitionRuntime;

use super::machine_instance_key::MachineInstanceKey;
use super::nested_machine_sample::normalized_machine_state_time;
use super::requests::StateMachineParameterProjection;
use super::state_machine_cache::{resolve_sub_machine_id, StateMachineEvaluationResult};
use super::AnimationEvaluationPipeline;
use crate::{CompiledAnimationStateMachine, TransitionDesc};

pub(super) struct ResolvedMachineInstance {
    pub(super) instance: MachineInstanceKey,
    pub(super) machine: Arc<CompiledAnimationStateMachine>,
    pub(super) evaluation: StateMachineEvaluationResult,
    pub(super) requested_desc: Option<TransitionDesc>,
    pub(super) requested_triggers: Option<Arc<[String]>>,
    pub(super) transition: Option<AnimationStateTransitionRuntime>,
    pub(super) root_active_state: String,
    pub(super) is_nested: bool,
}

pub(super) fn resolve_machine_instance(
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: &ProjectAssetManager,
    mut instance: MachineInstanceKey,
    mut machine_id: AssetId,
    mut active_state: Option<String>,
    mut transition: Option<AnimationStateTransitionRuntime>,
    parameters: StateMachineParameterProjection<'_>,
    skeleton_id: AssetId,
    time_seconds: zircon_runtime::core::math::Real,
) -> Option<ResolvedMachineInstance> {
    let mut root_active_state = None;
    let mut is_nested = false;
    loop {
        let (machine, evaluation, requested_desc, requested_triggers) = pipeline
            .evaluate_state_machine_with_triggers(
                asset_manager,
                &instance,
                machine_id,
                active_state.as_deref(),
                parameters,
            )?;
        root_active_state.get_or_insert_with(|| evaluation.active_state.clone());
        let requested_transition_ready = evaluation
            .transition
            .as_ref()
            .map(|transition| (evaluation.active_state.as_str(), transition))
            .zip(requested_desc)
            .is_some_and(|((state, _), desc)| {
                desc.exit_ready(normalized_machine_state_time(
                    pipeline,
                    asset_manager,
                    &instance,
                    &machine,
                    state,
                    parameters,
                    skeleton_id,
                    time_seconds,
                ))
            });
        if transition.is_some() || requested_transition_ready {
            return Some(ResolvedMachineInstance {
                instance,
                machine,
                evaluation,
                requested_desc,
                requested_triggers,
                transition,
                root_active_state: root_active_state.unwrap_or_default(),
                is_nested,
            });
        }
        let owner_state = evaluation.active_state.as_str();
        let Some(nested) = machine.sub_machine_for_state(owner_state) else {
            return Some(ResolvedMachineInstance {
                instance,
                machine,
                evaluation,
                requested_desc,
                requested_triggers,
                transition,
                root_active_state: root_active_state.unwrap_or_default(),
                is_nested,
            });
        };
        let nested_id = resolve_sub_machine_id(asset_manager, nested)?;
        instance = instance.nested(owner_state, nested_id)?;
        is_nested = true;
        machine_id = nested_id;
        active_state = pipeline.nested_machine_states.get(&instance).cloned();
        transition = pipeline.nested_machine_transitions.get(&instance).cloned();
    }
}
