use std::sync::Arc;

use zircon_runtime::asset::{
    AnimationGraphAsset, AnimationSkeletonAsset, AssetId, ProjectAssetManager,
};
use zircon_runtime::core::framework::animation::AnimationParameterMap;
use zircon_runtime::core::resource::{
    AnimationGraphMarker, AnimationSkeletonMarker, ResourceHandle, ResourceSnapshot,
};

use crate::{CompiledAnimationGraph, CompiledAnimationGraphEvaluation, SkeletonTargetTable};

use super::AnimationEvaluationPipeline;

const COMPILED_GRAPH_CACHE_LIMIT: usize = 128;

#[derive(Debug)]
pub(super) struct CachedCompiledGraph {
    pub(super) graph_revision: u64,
    pub(super) skeleton_revision: u64,
    pub(super) last_used: u64,
    pub(super) graph: Arc<CompiledAnimationGraph>,
}

impl AnimationEvaluationPipeline {
    pub(super) fn evaluate_graph(
        &mut self,
        assets: &ProjectAssetManager,
        graph_id: AssetId,
        skeleton_id: AssetId,
        parameters: &AnimationParameterMap,
    ) -> Option<CompiledAnimationGraphEvaluation> {
        let graph = load_graph_snapshot(assets, graph_id)?;
        let skeleton = load_skeleton_snapshot(assets, skeleton_id)?;
        let key = (graph_id, skeleton_id);
        self.graph_access_sequence = self.graph_access_sequence.saturating_add(1);
        let access = self.graph_access_sequence;

        let is_current = self.graph_cache.get(&key).is_some_and(|cached| {
            cached.graph_revision == graph.revision()
                && cached.skeleton_revision == skeleton.revision()
        });
        if !is_current {
            let targets = Arc::new(SkeletonTargetTable::compile(&skeleton).ok()?);
            let compiled = Arc::new(CompiledAnimationGraph::compile(&graph, targets).ok()?);
            self.graph_cache.insert(
                key,
                CachedCompiledGraph {
                    graph_revision: graph.revision(),
                    skeleton_revision: skeleton.revision(),
                    last_used: access,
                    graph: compiled,
                },
            );
            self.enforce_graph_cache_limit();
        }

        let cached = self.graph_cache.get_mut(&key)?;
        cached.last_used = access;
        Some(cached.graph.evaluate(parameters))
    }

    fn enforce_graph_cache_limit(&mut self) {
        while self.graph_cache.len() > COMPILED_GRAPH_CACHE_LIMIT {
            let Some(oldest) = self
                .graph_cache
                .iter()
                .min_by_key(|(key, cached)| (cached.last_used, **key))
                .map(|(key, _)| *key)
            else {
                break;
            };
            self.graph_cache.remove(&oldest);
        }
    }
}

fn load_graph_snapshot(
    assets: &ProjectAssetManager,
    id: AssetId,
) -> Option<ResourceSnapshot<AnimationGraphAsset>> {
    assets.load_animation_graph_asset(id).ok()?;
    assets
        .resource_manager()
        .snapshot::<AnimationGraphMarker, AnimationGraphAsset>(ResourceHandle::new(id))
}

fn load_skeleton_snapshot(
    assets: &ProjectAssetManager,
    id: AssetId,
) -> Option<ResourceSnapshot<AnimationSkeletonAsset>> {
    assets.load_animation_skeleton_asset(id).ok()?;
    assets
        .resource_manager()
        .snapshot::<AnimationSkeletonMarker, AnimationSkeletonAsset>(ResourceHandle::new(id))
}
