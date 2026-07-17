use std::sync::Arc;

use crate::core::framework::render::{IblBakeArtifactRequest, RenderPipelineHandle};
use crate::graphics::feature::RenderFeatureCapabilityRequirement;
use crate::render_graph::CompiledRenderGraph;

use crate::graphics::extract::FrameHistoryBinding;

use super::{render_pass_stage::RenderPassStage, renderer_feature_asset::RendererFeatureAsset};

mod resource_write_index;
mod runtime_feature_flags;
mod runtime_metadata;

pub(crate) use runtime_feature_flags::CompiledRenderPipelineRuntimeFeatureFlags;
pub(crate) use runtime_metadata::CompiledRenderPipelineRuntimeMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPipelinePassStage {
    pub pass_name: String,
    pub stage: RenderPassStage,
}

impl CompiledRenderPipelinePassStage {
    pub fn new(pass_name: impl Into<String>, stage: RenderPassStage) -> Self {
        Self {
            pass_name: pass_name.into(),
            stage,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPipeline {
    pub handle: RenderPipelineHandle,
    pub name: String,
    pub renderer_name: String,
    pub stages: Vec<RenderPassStage>,
    pub pass_stages: Vec<CompiledRenderPipelinePassStage>,
    enabled_features: Vec<RendererFeatureAsset>,
    pub required_extract_sections: Vec<String>,
    pub capability_requirements: Vec<RenderFeatureCapabilityRequirement>,
    pub history_bindings: Vec<FrameHistoryBinding>,
    pub environment_ibl_bake_request: Option<IblBakeArtifactRequest>,
    runtime_metadata: Arc<CompiledRenderPipelineRuntimeMetadata>,
    graph: CompiledRenderGraph,
}

pub(crate) struct CompiledRenderPipelineParts {
    pub(crate) handle: RenderPipelineHandle,
    pub(crate) name: String,
    pub(crate) renderer_name: String,
    pub(crate) stages: Vec<RenderPassStage>,
    pub(crate) pass_stages: Vec<CompiledRenderPipelinePassStage>,
    pub(crate) enabled_features: Vec<RendererFeatureAsset>,
    pub(crate) required_extract_sections: Vec<String>,
    pub(crate) capability_requirements: Vec<RenderFeatureCapabilityRequirement>,
    pub(crate) history_bindings: Vec<FrameHistoryBinding>,
    pub(crate) environment_ibl_bake_request: Option<IblBakeArtifactRequest>,
    pub(crate) graph: CompiledRenderGraph,
}

impl CompiledRenderPipeline {
    pub(crate) fn from_parts(parts: CompiledRenderPipelineParts) -> Self {
        let runtime_metadata =
            Arc::new(CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(
                &parts.enabled_features,
                &parts.capability_requirements,
                &parts.graph,
            ));
        Self {
            handle: parts.handle,
            name: parts.name,
            renderer_name: parts.renderer_name,
            stages: parts.stages,
            pass_stages: parts.pass_stages,
            enabled_features: parts.enabled_features,
            required_extract_sections: parts.required_extract_sections,
            capability_requirements: parts.capability_requirements,
            history_bindings: parts.history_bindings,
            environment_ibl_bake_request: parts.environment_ibl_bake_request,
            runtime_metadata,
            graph: parts.graph,
        }
    }

    pub fn enabled_features(&self) -> &[RendererFeatureAsset] {
        &self.enabled_features
    }

    pub fn graph(&self) -> &CompiledRenderGraph {
        &self.graph
    }

    pub fn pass_stage(&self, pass_name: &str) -> Option<RenderPassStage> {
        self.pass_stages
            .iter()
            .find(|entry| entry.pass_name == pass_name)
            .map(|entry| entry.stage)
    }

    pub fn writes_resource(&self, resource_name: &str) -> bool {
        self.runtime_metadata.writes_resource(resource_name)
    }

    pub(crate) fn runtime_feature_flags(&self) -> CompiledRenderPipelineRuntimeFeatureFlags {
        self.runtime_metadata.runtime_feature_flags()
    }

    pub(crate) fn executor_validation_generation(&self) -> u64 {
        self.runtime_metadata.validation_generation()
    }
}

#[cfg(test)]
mod tests;
