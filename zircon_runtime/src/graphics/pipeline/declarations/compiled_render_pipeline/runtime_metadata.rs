use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, OnceLock,
};

use crate::graphics::{RenderFeatureCapabilityRequirement, RendererFeatureAsset};
use crate::render_graph::CompiledRenderGraph;

use super::resource_write_index::CompiledRenderPipelineResourceWriteIndex;
use super::runtime_feature_flags::CompiledRenderPipelineRuntimeFeatureFlags;

static NEXT_VALIDATION_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub(crate) struct CompiledRenderPipelineRuntimeMetadata {
    runtime_feature_flags: CompiledRenderPipelineRuntimeFeatureFlags,
    resource_write_index: CompiledRenderPipelineResourceWriteIndex,
    graph_dump_text: OnceLock<Arc<str>>,
    validation_generation: u64,
}

impl PartialEq for CompiledRenderPipelineRuntimeMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.runtime_feature_flags == other.runtime_feature_flags
            && self.resource_write_index == other.resource_write_index
    }
}

impl Eq for CompiledRenderPipelineRuntimeMetadata {}

impl CompiledRenderPipelineRuntimeMetadata {
    pub(crate) fn from_compiled_inputs(
        enabled_features: &[RendererFeatureAsset],
        capability_requirements: &[RenderFeatureCapabilityRequirement],
        graph: &CompiledRenderGraph,
    ) -> Self {
        Self {
            runtime_feature_flags: CompiledRenderPipelineRuntimeFeatureFlags::from_compiled_inputs(
                enabled_features,
                capability_requirements,
            ),
            resource_write_index: CompiledRenderPipelineResourceWriteIndex::from_graph(graph),
            graph_dump_text: OnceLock::new(),
            validation_generation: next_validation_generation(),
        }
    }

    pub(super) fn runtime_feature_flags(&self) -> CompiledRenderPipelineRuntimeFeatureFlags {
        self.runtime_feature_flags
    }

    pub(super) fn writes_resource(&self, resource_name: &str) -> bool {
        self.resource_write_index.contains(resource_name)
    }

    pub(super) fn graph_dump_text(&self, graph: &CompiledRenderGraph) -> Arc<str> {
        Arc::clone(
            self.graph_dump_text
                .get_or_init(|| Arc::from(graph.dump().to_text())),
        )
    }

    pub(super) const fn validation_generation(&self) -> u64 {
        self.validation_generation
    }

    #[cfg(test)]
    pub(super) fn build_stats(&self) -> (usize, usize) {
        self.resource_write_index.build_stats()
    }

    #[cfg(test)]
    pub(super) fn resource_write_storage_snapshot(&self) -> (usize, usize, usize) {
        self.resource_write_index.storage_snapshot()
    }
}

fn next_validation_generation() -> u64 {
    let generation = NEXT_VALIDATION_GENERATION.fetch_add(1, Ordering::Relaxed);
    if generation == 0 {
        NEXT_VALIDATION_GENERATION.fetch_add(1, Ordering::Relaxed)
    } else {
        generation
    }
}
