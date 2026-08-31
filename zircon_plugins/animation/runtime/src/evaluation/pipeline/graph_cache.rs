use std::sync::Arc;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationParameterSet, AnimationSkeletonAsset,
};
use zircon_runtime::core::resource::{
    AnimationGraphMarker, AnimationSkeletonMarker, ResourceHandle, ResourceSnapshot,
};

use crate::{
    compile_animation_graph_runtime, CompiledAnimationGraph, CompiledAnimationGraphEvaluation,
    SkeletonTargetTable,
};

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
        parameters: &AnimationParameterSet,
    ) -> Option<Arc<CompiledAnimationGraphEvaluation>> {
        let cache_key = (graph_id, skeleton_id, parameters.content_fingerprint());
        if let Some(cached) = self
            .graph_evaluation_cache
            .get(&cache_key)
            .filter(|cached| cached.parameters == *parameters)
        {
            return Some(Arc::clone(&cached.evaluation));
        }
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
            let compiled = Arc::new(compile_animation_graph_runtime(&graph, targets).ok()?);
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
        let evaluation = Arc::new(cached.graph.evaluate(parameters.as_map()));
        self.graph_evaluation_count = self.graph_evaluation_count.saturating_add(1);
        self.cache_graph_evaluation(graph_id, skeleton_id, parameters, Arc::clone(&evaluation));
        Some(evaluation)
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
    let resources = assets.resource_manager();
    let handle = ResourceHandle::<AnimationGraphMarker>::new(id);
    resources.snapshot(handle).or_else(|| {
        assets.load_animation_graph_asset(id).ok()?;
        resources.snapshot(handle)
    })
}

fn load_skeleton_snapshot(
    assets: &ProjectAssetManager,
    id: AssetId,
) -> Option<ResourceSnapshot<AnimationSkeletonAsset>> {
    let resources = assets.resource_manager();
    let handle = ResourceHandle::<AnimationSkeletonMarker>::new(id);
    resources.snapshot(handle).or_else(|| {
        assets.load_animation_skeleton_asset(id).ok()?;
        resources.snapshot(handle)
    })
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830cg_graph_cache_checks_resident_snapshots_first() {
        let source = include_str!("graph_cache.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        for (start, end, loader) in [
            (
                "fn load_graph_snapshot(",
                "fn load_skeleton_snapshot(",
                "load_animation_graph_asset",
            ),
            (
                "fn load_skeleton_snapshot(",
                "#[cfg(test)]",
                "load_animation_skeleton_asset",
            ),
        ] {
            let start = source.find(start).expect("snapshot helper");
            let helper = production.get(start..).unwrap_or(&source[start..]);
            let helper = helper.split(end).next().expect("snapshot helper boundary");
            assert!(
                helper.find("resources.snapshot").expect("resident lookup")
                    < helper.find(loader).expect("loader fallback")
            );
            assert!(helper.contains(".or_else(||"));
        }
    }
}
