use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    AnimationParameterRevision, AnimationParameterSet, AnimationParameterValue,
    AnimationStateMachineAsset, AnimationStateTransitionEvaluation,
};
use zircon_runtime::core::resource::{
    AnimationStateMachineMarker, ResourceHandle, ResourceSnapshot,
};
use zircon_runtime::scene::EntityId;

use crate::state_machine::{
    compile_animation_state_machine_runtime_bundle, StateMachineBlendSamplingState,
};
use crate::{
    CompiledAnimationStateMachine, CompiledStateMachineEvaluation, CompiledStateMachineLayers,
    TransitionDesc,
};

use super::requests::StateMachineParameterProjection;
use super::{machine_instance_key::MachineInstanceKey, AnimationEvaluationPipeline};

const COMPILED_STATE_MACHINE_CACHE_LIMIT: usize = 64;
const STATE_MACHINE_INSTANCE_CACHE_LIMIT: usize = 4_096;

#[derive(Debug)]
pub(super) struct CachedCompiledStateMachine {
    revision: u64,
    last_used: u64,
    machine: Arc<CompiledAnimationStateMachine>,
    layers: Arc<CompiledStateMachineLayers>,
}

#[derive(Debug)]
pub(super) struct StateMachineEvaluationResult {
    pub(super) active_state: String,
    pub(super) transition: Option<AnimationStateTransitionEvaluation>,
}

#[derive(Debug)]
struct CachedStateMachineInstance {
    referenced: bool,
    sampling: StateMachineBlendSamplingState,
    parameter_layout: Arc<[String]>,
    parameter_revision: AnimationParameterRevision,
    parameter_values: Box<[Option<AnimationParameterValue>]>,
}

impl CachedStateMachineInstance {
    fn refresh(
        &mut self,
        machine: &CompiledAnimationStateMachine,
        parameters: StateMachineParameterProjection<'_>,
    ) {
        self.sampling.ensure_state_count(machine.state_count());
        let layout_is_current = Arc::ptr_eq(&self.parameter_layout, machine.parameter_layout());
        if !layout_is_current || self.parameter_revision != parameters.revision {
            self.parameter_layout = Arc::clone(machine.parameter_layout());
            self.parameter_revision = parameters.revision;
            self.parameter_values = machine.project_parameters(parameters.values);
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct StateMachineInstanceCache {
    entries: BTreeMap<MachineInstanceKey, CachedStateMachineInstance>,
    // New entries are probationary; a later hit grants one second chance at eviction.
    eviction_clock: VecDeque<MachineInstanceKey>,
}

impl StateMachineInstanceCache {
    fn state_for(
        &mut self,
        instance: &MachineInstanceKey,
        machine: &CompiledAnimationStateMachine,
        parameters: StateMachineParameterProjection<'_>,
    ) -> &mut CachedStateMachineInstance {
        if self.entries.contains_key(instance) {
            {
                let cached = self
                    .entries
                    .get_mut(instance)
                    .expect("instance cache entry exists after membership check");
                cached.referenced = true;
                cached.refresh(machine, parameters);
            }
            return self
                .entries
                .get_mut(instance)
                .expect("instance cache entry remains after refresh");
        }
        while self.entries.len() >= STATE_MACHINE_INSTANCE_CACHE_LIMIT {
            let candidate = self
                .eviction_clock
                .pop_front()
                .expect("a full instance cache has an eviction clock entry");
            let cached = self
                .entries
                .get_mut(&candidate)
                .expect("eviction clock points to a cached instance");
            if cached.referenced {
                cached.referenced = false;
                self.eviction_clock.push_back(candidate);
                continue;
            }
            self.entries
                .remove(&candidate)
                .expect("cold eviction candidate remains cached");
        }
        self.entries.insert(
            instance.clone(),
            CachedStateMachineInstance {
                referenced: false,
                sampling: StateMachineBlendSamplingState::new(machine.state_count()),
                parameter_layout: Arc::clone(machine.parameter_layout()),
                parameter_revision: parameters.revision,
                parameter_values: machine.project_parameters(parameters.values),
            },
        );
        self.eviction_clock.push_back(instance.clone());
        let cached = self
            .entries
            .get_mut(instance)
            .expect("inserted or existing sampling cache entry");
        cached
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.eviction_clock.clear();
    }

    pub(super) fn retain_entities(&mut self, active: &BTreeSet<EntityId>) {
        self.entries
            .retain(|instance, _| active.contains(&instance.entity()));
        self.eviction_clock
            .retain(|instance| active.contains(&instance.entity()));
    }
}

impl AnimationEvaluationPipeline {
    pub(super) fn evaluate_state_machine(
        &mut self,
        assets: &ProjectAssetManager,
        instance: &MachineInstanceKey,
        id: AssetId,
        current_state: Option<&str>,
        parameters: StateMachineParameterProjection<'_>,
    ) -> Option<(
        Arc<CompiledAnimationStateMachine>,
        StateMachineEvaluationResult,
        Option<TransitionDesc>,
    )> {
        let (machine, evaluation, transition_desc, _) = self.evaluate_state_machine_internal(
            assets,
            instance,
            id,
            current_state,
            parameters,
            false,
        )?;
        Some((machine, evaluation, transition_desc))
    }

    pub(super) fn evaluate_state_machine_with_triggers(
        &mut self,
        assets: &ProjectAssetManager,
        instance: &MachineInstanceKey,
        id: AssetId,
        current_state: Option<&str>,
        parameters: StateMachineParameterProjection<'_>,
    ) -> Option<(
        Arc<CompiledAnimationStateMachine>,
        StateMachineEvaluationResult,
        Option<TransitionDesc>,
        Option<Arc<[String]>>,
    )> {
        self.evaluate_state_machine_internal(assets, instance, id, current_state, parameters, true)
    }

    fn evaluate_state_machine_internal(
        &mut self,
        assets: &ProjectAssetManager,
        instance: &MachineInstanceKey,
        id: AssetId,
        current_state: Option<&str>,
        parameters: StateMachineParameterProjection<'_>,
        include_triggers: bool,
    ) -> Option<(
        Arc<CompiledAnimationStateMachine>,
        StateMachineEvaluationResult,
        Option<TransitionDesc>,
        Option<Arc<[String]>>,
    )> {
        self.ensure_state_machine_cached(assets, id)?;
        let cached = self.state_machine_cache.get(&id)?;
        let machine = Arc::clone(&cached.machine);
        let evaluated = self.evaluate_compiled_state_machine_with_sampling(
            instance,
            &machine,
            current_state,
            parameters,
        );
        let transition = evaluated.transition().cloned();
        let transition_desc = evaluated.transition_desc();
        let consumed_triggers = include_triggers
            .then(|| evaluated.shared_consumed_triggers())
            .flatten();
        let active_state = evaluated.active_state().to_string();
        Some((
            machine,
            StateMachineEvaluationResult {
                active_state,
                transition,
            },
            transition_desc,
            consumed_triggers,
        ))
    }

    pub(super) fn evaluate_compiled_state_machine_with_sampling<'a>(
        &mut self,
        instance: &MachineInstanceKey,
        machine: &'a CompiledAnimationStateMachine,
        current_state: Option<&str>,
        parameters: StateMachineParameterProjection<'_>,
    ) -> CompiledStateMachineEvaluation<'a> {
        let state = self
            .state_machine_instance_cache
            .state_for(instance, machine, parameters);
        machine.evaluate_with_blend_sampling(
            current_state,
            &state.parameter_values,
            &mut state.sampling,
        )
    }

    pub(super) fn graph_samples_for_state_with_sampling<'a>(
        &mut self,
        instance: &MachineInstanceKey,
        machine: &'a CompiledAnimationStateMachine,
        state_name: &str,
        parameters: StateMachineParameterProjection<'_>,
    ) -> Option<crate::state_machine::CompiledGraphSamples<'a>> {
        let state = self
            .state_machine_instance_cache
            .state_for(instance, machine, parameters);
        machine.graph_samples_for_state_with_blend_sampling(
            state_name,
            &state.parameter_values,
            &mut state.sampling,
        )
    }

    fn ensure_state_machine_cached(
        &mut self,
        assets: &ProjectAssetManager,
        id: AssetId,
    ) -> Option<()> {
        let source = load_state_machine_snapshot(assets, id)?;
        self.state_machine_access_sequence = self.state_machine_access_sequence.saturating_add(1);
        let access = self.state_machine_access_sequence;
        let is_current = self
            .state_machine_cache
            .get(&id)
            .is_some_and(|cached| cached.revision == source.revision());
        if !is_current {
            let (machine, layers) = compile_animation_state_machine_runtime_bundle(&source).ok()?;
            self.state_machine_cache.insert(
                id,
                CachedCompiledStateMachine {
                    revision: source.revision(),
                    last_used: access,
                    machine: Arc::new(machine),
                    layers: Arc::new(layers),
                },
            );
            self.enforce_state_machine_cache_limit();
        }
        let cached = self.state_machine_cache.get_mut(&id)?;
        cached.last_used = access;
        Some(())
    }

    fn enforce_state_machine_cache_limit(&mut self) {
        while self.state_machine_cache.len() > COMPILED_STATE_MACHINE_CACHE_LIMIT {
            let oldest = self
                .state_machine_cache
                .iter()
                .min_by_key(|(id, cached)| (cached.last_used, **id))
                .map(|(id, _)| *id);
            let Some(oldest) = oldest else { break };
            self.state_machine_cache.remove(&oldest);
        }
    }

    pub(super) fn compiled_state_machine_layers(
        &mut self,
        assets: &ProjectAssetManager,
        id: AssetId,
    ) -> Option<Arc<CompiledStateMachineLayers>> {
        self.ensure_state_machine_cached(assets, id)?;
        self.state_machine_cache
            .get(&id)
            .map(|cached| Arc::clone(&cached.layers))
    }
}

fn load_state_machine_snapshot(
    assets: &ProjectAssetManager,
    id: AssetId,
) -> Option<ResourceSnapshot<AnimationStateMachineAsset>> {
    assets.load_animation_state_machine_asset(id).ok()?;
    assets
        .resource_manager()
        .snapshot::<AnimationStateMachineMarker, AnimationStateMachineAsset>(ResourceHandle::new(
            id,
        ))
}

pub(super) fn resolve_sub_machine_id(
    assets: &ProjectAssetManager,
    machine: &zircon_runtime::asset::AssetReference,
) -> Option<AssetId> {
    let id = assets.resolve_asset_id(&machine.locator)?;
    load_state_machine_snapshot(assets, id)?;
    Some(id)
}

#[cfg(test)]
#[path = "tests/state_machine_cache.rs"]
mod tests;
