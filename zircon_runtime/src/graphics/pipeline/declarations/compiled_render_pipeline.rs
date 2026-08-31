use std::sync::Arc;

use crate::core::framework::render::{IblBakeArtifactRequest, RenderPipelineHandle};
use crate::graphics::feature::RenderFeatureCapabilityRequirement;

use crate::graphics::extract::FrameHistoryBinding;

use super::{
    ambient_occlusion::{AmbientOcclusionOutputs, CompiledAoProfile},
    render_pass_stage::RenderPassStage,
    renderer_feature_asset::RendererFeatureAsset,
};

mod execution_packet;
mod history_epilogue_plan;
mod resource_write_index;
mod runtime_feature_flags;
mod runtime_metadata;

pub(crate) use execution_packet::{
    RenderGraphExecutionBatch, RenderGraphExecutionCursor, RenderGraphExecutionPacket,
    RenderGraphExecutionPass, RenderGraphExecutionPassMetadata,
};
pub(crate) use history_epilogue_plan::{CompiledHistoryEpiloguePlan, CompiledHistoryTextureSource};
pub(crate) use runtime_feature_flags::CompiledRenderPipelineRuntimeFeatureFlags;
pub(crate) use runtime_metadata::CompiledRenderPipelineRuntimeMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPipeline {
    pub handle: RenderPipelineHandle,
    pub name: String,
    pub renderer_name: String,
    enabled_features: Vec<RendererFeatureAsset>,
    pub required_extract_sections: Vec<String>,
    pub capability_requirements: Vec<RenderFeatureCapabilityRequirement>,
    pub history_bindings: Vec<FrameHistoryBinding>,
    pub environment_ibl_bake_request: Option<IblBakeArtifactRequest>,
    ambient_occlusion_profile: Option<CompiledAoProfile>,
    ambient_occlusion_outputs: Option<AmbientOcclusionOutputs>,
    half_resolution_transparency_depth_sigma: u16,
    runtime_metadata: Arc<CompiledRenderPipelineRuntimeMetadata>,
    execution_packet: RenderGraphExecutionPacket,
    history_epilogue_plan: CompiledHistoryEpiloguePlan,
}

pub(crate) struct CompiledRenderPipelineParts {
    pub(crate) handle: RenderPipelineHandle,
    pub(crate) name: String,
    pub(crate) renderer_name: String,
    pub(crate) execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    pub(crate) enabled_features: Vec<RendererFeatureAsset>,
    pub(crate) required_extract_sections: Vec<String>,
    pub(crate) capability_requirements: Vec<RenderFeatureCapabilityRequirement>,
    pub(crate) history_bindings: Vec<FrameHistoryBinding>,
    pub(crate) environment_ibl_bake_request: Option<IblBakeArtifactRequest>,
    pub(crate) ambient_occlusion_profile: Option<CompiledAoProfile>,
    pub(crate) half_resolution_transparency_depth_sigma: u16,
    pub(crate) graph: crate::render_graph::CompiledRenderGraph,
}

impl CompiledRenderPipeline {
    pub(crate) fn from_parts(parts: CompiledRenderPipelineParts) -> Result<Self, String> {
        let execution_packet =
            RenderGraphExecutionPacket::new(parts.graph, parts.execution_pass_metadata)?;
        let history_epilogue_plan =
            CompiledHistoryEpiloguePlan::from_graph(execution_packet.graph())?;
        let runtime_metadata =
            Arc::new(CompiledRenderPipelineRuntimeMetadata::from_compiled_inputs(
                &parts.enabled_features,
                &parts.capability_requirements,
                execution_packet.graph(),
            ));
        let ambient_occlusion_profile = parts.ambient_occlusion_profile.map(|profile| {
            profile.with_pipeline_generation(runtime_metadata.validation_generation())
        });
        let ambient_occlusion_outputs = ambient_occlusion_profile
            .as_ref()
            .map(|profile| {
                AmbientOcclusionOutputs::from_compiled_graph(profile, execution_packet.graph())
            })
            .transpose()?;
        Ok(Self {
            handle: parts.handle,
            name: parts.name,
            renderer_name: parts.renderer_name,
            enabled_features: parts.enabled_features,
            required_extract_sections: parts.required_extract_sections,
            capability_requirements: parts.capability_requirements,
            history_bindings: parts.history_bindings,
            environment_ibl_bake_request: parts.environment_ibl_bake_request,
            ambient_occlusion_profile,
            ambient_occlusion_outputs,
            half_resolution_transparency_depth_sigma: parts
                .half_resolution_transparency_depth_sigma,
            runtime_metadata,
            execution_packet,
            history_epilogue_plan,
        })
    }

    pub fn enabled_features(&self) -> &[RendererFeatureAsset] {
        &self.enabled_features
    }

    pub fn graph(&self) -> &crate::render_graph::CompiledRenderGraph {
        self.execution_packet.graph()
    }

    /// Builds graph diagnostics at most once for this immutable compiled generation.
    pub(in crate::graphics) fn graph_dump_text(&self) -> Arc<str> {
        self.runtime_metadata.graph_dump_text(self.graph())
    }

    pub fn pass_stage(&self, pass_name: &str) -> Option<RenderPassStage> {
        self.execution_packet.stage_for_pass_name(pass_name)
    }

    pub(in crate::graphics) fn execution_passes_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> impl Iterator<Item = &RenderGraphExecutionPass> {
        self.execution_packet.passes_for_stage(stage)
    }

    pub(in crate::graphics) fn execution_passes_in_graph_order(
        &self,
    ) -> impl Iterator<Item = &RenderGraphExecutionPass> {
        self.execution_packet.execution_passes_in_graph_order()
    }

    pub(in crate::graphics) fn execution_batches(
        &self,
    ) -> impl Iterator<Item = &RenderGraphExecutionBatch> {
        self.execution_packet.execution_batches()
    }

    pub(in crate::graphics) fn execution_batches_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> impl Iterator<Item = &RenderGraphExecutionBatch> {
        self.execution_packet.execution_batches_for_stage(stage)
    }

    pub(in crate::graphics) fn execution_batches_with_indices_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> impl Iterator<Item = (usize, &RenderGraphExecutionBatch)> {
        self.execution_packet
            .execution_batches_with_indices_for_stage(stage)
    }

    pub(in crate::graphics) fn execution_batch_index_for_pass(
        &self,
        graph_pass_index: usize,
    ) -> Option<usize> {
        self.execution_packet
            .execution_batch_index_for_pass(graph_pass_index)
    }

    pub(in crate::graphics) fn execution_stages_in_graph_order(
        &self,
    ) -> impl Iterator<Item = RenderPassStage> + '_ {
        self.execution_packet.execution_stages_in_graph_order()
    }

    pub(in crate::graphics) fn execution_passes_for_batch(
        &self,
        batch: &RenderGraphExecutionBatch,
    ) -> impl Iterator<Item = &RenderGraphExecutionPass> {
        self.execution_packet.passes_for_batch(batch)
    }

    pub(in crate::graphics) const fn begin_execution(&self) -> RenderGraphExecutionCursor {
        self.execution_packet.begin_execution()
    }

    pub(in crate::graphics) fn admit_execution_pass(
        &self,
        cursor: &mut RenderGraphExecutionCursor,
        graph_pass_index: usize,
    ) -> Result<(), String> {
        self.execution_packet
            .admit_execution_pass(cursor, graph_pass_index)
    }

    pub(in crate::graphics) fn finish_execution(
        &self,
        cursor: RenderGraphExecutionCursor,
    ) -> Result<(), String> {
        self.execution_packet.finish_execution(cursor)
    }

    pub(in crate::graphics) const fn execution_batch_report(
        &self,
    ) -> crate::core::framework::render::RenderGraphExecutionBatchReport {
        self.execution_packet.execution_batch_report()
    }

    pub(in crate::graphics) const fn history_epilogue_plan(&self) -> &CompiledHistoryEpiloguePlan {
        &self.history_epilogue_plan
    }

    pub(in crate::graphics) fn execution_access_ids_for_pass(
        &self,
        graph_pass_index: usize,
    ) -> Option<&[crate::render_graph::RenderGraphResourceAccessId]> {
        self.execution_packet.access_ids_for_pass(graph_pass_index)
    }

    pub fn writes_resource(&self, resource_name: &str) -> bool {
        self.runtime_metadata.writes_resource(resource_name)
    }

    pub fn ambient_occlusion_profile(&self) -> Option<&CompiledAoProfile> {
        self.ambient_occlusion_profile.as_ref()
    }

    pub fn ambient_occlusion_outputs(&self) -> Option<&AmbientOcclusionOutputs> {
        self.ambient_occlusion_outputs.as_ref()
    }

    pub(crate) fn runtime_feature_flags(&self) -> CompiledRenderPipelineRuntimeFeatureFlags {
        self.runtime_metadata.runtime_feature_flags()
    }

    pub(crate) const fn half_resolution_transparency_depth_sigma(&self) -> u16 {
        self.half_resolution_transparency_depth_sigma
    }

    pub(crate) fn executor_validation_generation(&self) -> u64 {
        self.runtime_metadata.validation_generation()
    }
}

#[cfg(test)]
mod tests;
