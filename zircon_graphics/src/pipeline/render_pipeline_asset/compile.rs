use std::collections::{BTreeMap, BTreeSet};

use zircon_framework::render::RenderFrameExtract;
use zircon_render_graph::RenderGraphBuilder;

use crate::extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot};
use crate::pipeline::declarations::{
    CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions,
};

use super::super::validation::{stage_pass_descriptors, validate_renderer_asset};

impl RenderPipelineAsset {
    pub fn compile(&self, extract: &RenderFrameExtract) -> Result<CompiledRenderPipeline, String> {
        self.compile_with_options(extract, &RenderPipelineCompileOptions::default())
    }

    pub fn compile_with_options(
        &self,
        extract: &RenderFrameExtract,
        options: &RenderPipelineCompileOptions,
    ) -> Result<CompiledRenderPipeline, String> {
        let _ = extract;
        validate_renderer_asset(&self.renderer)?;
        let enabled_features = self
            .renderer
            .features
            .iter()
            .filter(|feature| feature.enabled && options.permits_feature(feature.feature))
            .cloned()
            .collect::<Vec<_>>();
        let enabled_descriptors = enabled_features
            .iter()
            .map(|feature| feature.feature.descriptor())
            .collect::<Vec<_>>();

        let mut required_extract_sections = BTreeSet::new();
        let mut history_access_by_slot = BTreeMap::<FrameHistorySlot, FrameHistoryAccess>::new();
        for descriptor in &enabled_descriptors {
            for section in &descriptor.required_extract_sections {
                required_extract_sections.insert(section.clone());
            }
            for binding in &descriptor.history_bindings {
                history_access_by_slot
                    .entry(binding.slot)
                    .and_modify(|access| *access = access.merge(binding.access))
                    .or_insert(binding.access);
            }
        }
        let history_bindings = history_access_by_slot
            .into_iter()
            .map(|(slot, access)| FrameHistoryBinding { slot, access })
            .collect::<Vec<_>>();

        let mut graph = RenderGraphBuilder::new(self.name.clone());
        let mut previous = None;
        for stage in &self.renderer.stages {
            for pass_descriptor in stage_pass_descriptors(*stage, &enabled_descriptors) {
                let pass = graph.add_pass(
                    pass_descriptor.pass_name.clone(),
                    options.resolve_queue(pass_descriptor.queue),
                );
                if let Some(before) = previous {
                    graph
                        .add_dependency(before, pass)
                        .map_err(|error| error.to_string())?;
                }
                previous = Some(pass);
            }
        }

        Ok(CompiledRenderPipeline {
            handle: self.handle,
            name: self.name.clone(),
            renderer_name: self.renderer.name.clone(),
            stages: self.renderer.stages.clone(),
            enabled_features,
            required_extract_sections: required_extract_sections.into_iter().collect(),
            history_bindings,
            graph: graph.compile().map_err(|error| error.to_string())?,
        })
    }
}
