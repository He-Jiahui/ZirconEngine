use std::sync::Arc;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::AnimationStateMachineAsset;
use zircon_runtime::core::framework::animation::{
    AnimationParameterMap, AnimationStateMachineEvaluation,
};
use zircon_runtime::core::resource::{
    AnimationStateMachineMarker, ResourceHandle, ResourceSnapshot,
};

use crate::{CompiledAnimationStateMachine, CompiledStateMachineLayers, TransitionDesc};

use super::AnimationEvaluationPipeline;

const COMPILED_STATE_MACHINE_CACHE_LIMIT: usize = 64;

#[derive(Debug)]
pub(super) struct CachedCompiledStateMachine {
    revision: u64,
    last_used: u64,
    machine: Arc<CompiledAnimationStateMachine>,
    layers: Arc<CompiledStateMachineLayers>,
}

impl AnimationEvaluationPipeline {
    pub(super) fn evaluate_state_machine(
        &mut self,
        assets: &ProjectAssetManager,
        id: AssetId,
        current_state: Option<&str>,
        parameters: &AnimationParameterMap,
    ) -> Option<(
        Arc<CompiledAnimationStateMachine>,
        AnimationStateMachineEvaluation,
        Option<TransitionDesc>,
    )> {
        self.ensure_state_machine_cached(assets, id)?;
        let cached = self.state_machine_cache.get(&id)?;
        let machine = Arc::clone(&cached.machine);
        let evaluated = machine.evaluate(current_state, parameters);
        let transition = evaluated.transition().cloned();
        let transition_desc = evaluated.transition_desc();
        let active_state = evaluated.active_state().to_string();
        let graph = evaluated
            .graph_samples()
            .next()
            .map(|(graph, _)| graph.clone());
        Some((
            machine,
            AnimationStateMachineEvaluation {
                parameters: parameters.clone(),
                active_state: Some(active_state),
                transitioned: false,
                graph,
                transition,
            },
            transition_desc,
        ))
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
            self.state_machine_cache.insert(
                id,
                CachedCompiledStateMachine {
                    revision: source.revision(),
                    last_used: access,
                    machine: Arc::new(CompiledAnimationStateMachine::compile(&source).ok()?),
                    layers: Arc::new(CompiledStateMachineLayers::compile(&source).ok()?),
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
