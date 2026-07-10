use std::collections::BTreeMap;

use zircon_runtime::scene::ecs::Resource;

use crate::{AnimationClipEvaluator, AnimationClipEvaluatorStats};

use super::graph_cache::CachedCompiledGraph;

/// Persistent state shared by every phase of `animation.evaluate`.
#[derive(Debug, Default)]
pub struct AnimationEvaluationPipeline {
    clip_evaluator: AnimationClipEvaluator,
    pub(super) graph_cache: BTreeMap<
        (
            zircon_runtime::asset::AssetId,
            zircon_runtime::asset::AssetId,
        ),
        CachedCompiledGraph,
    >,
    pub(super) graph_access_sequence: u64,
}

impl AnimationEvaluationPipeline {
    pub fn clip_evaluator_stats(&self) -> AnimationClipEvaluatorStats {
        self.clip_evaluator.stats()
    }

    pub fn compiled_graph_cache_len(&self) -> usize {
        self.graph_cache.len()
    }

    pub(super) fn clip_evaluator_mut(&mut self) -> &mut AnimationClipEvaluator {
        &mut self.clip_evaluator
    }
}

impl Resource for AnimationEvaluationPipeline {}
