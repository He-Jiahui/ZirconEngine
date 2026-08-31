use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderGraphAccessAllocationBinding, RenderGraphResourceAccessId,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
};
use crate::rhi::{TextureDesc, TextureUsage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompiledHistoryTextureSource {
    access_id: RenderGraphResourceAccessId,
    desc: TextureDesc,
}

impl CompiledHistoryTextureSource {
    pub(crate) const fn access_id(&self) -> RenderGraphResourceAccessId {
        self.access_id
    }

    pub(crate) const fn desc(&self) -> &TextureDesc {
        &self.desc
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CompiledHistoryEpiloguePlan {
    hybrid_global_illumination: Option<CompiledHistoryTextureSource>,
    global_illumination: Option<CompiledHistoryTextureSource>,
    global_illumination_temporal_metadata: Option<CompiledHistoryTextureSource>,
    screen_space_reflection: Option<CompiledHistoryTextureSource>,
    hzb_furthest: Option<CompiledHistoryTextureSource>,
    volumetric_scattering: Option<CompiledHistoryTextureSource>,
}

impl CompiledHistoryEpiloguePlan {
    pub(super) fn from_graph(graph: &CompiledRenderGraph) -> Result<Self, String> {
        let mut plan = Self::default();
        for binding in graph.access_allocation_bindings() {
            if binding.key.access != RenderGraphResourceAccessKind::Write {
                continue;
            }
            let declaration = graph
                .resource_declaration(binding.key.resource)
                .ok_or_else(|| {
                    format!(
                        "history epilogue access {:?} references an undeclared graph resource",
                        binding.key.access_id
                    )
                })?;
            let destination = match declaration.name.as_str() {
                PostProcessGraphResourceNames::HYBRID_GI_LIGHTING => {
                    &mut plan.hybrid_global_illumination
                }
                PostProcessGraphResourceNames::GLOBAL_ILLUMINATION => &mut plan.global_illumination,
                PostProcessGraphResourceNames::HYBRID_GI_TEMPORAL_METADATA => {
                    &mut plan.global_illumination_temporal_metadata
                }
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY => {
                    &mut plan.screen_space_reflection
                }
                PostProcessGraphResourceNames::HZB_FURTHEST => &mut plan.hzb_furthest,
                PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING => {
                    &mut plan.volumetric_scattering
                }
                _ => continue,
            };
            *destination = Some(history_texture_source(graph, binding)?);
        }
        Ok(plan)
    }

    pub(crate) fn global_illumination_sources(
        &self,
    ) -> impl Iterator<Item = &CompiledHistoryTextureSource> {
        [
            self.hybrid_global_illumination.as_ref(),
            self.global_illumination.as_ref(),
        ]
        .into_iter()
        .flatten()
    }

    pub(crate) const fn global_illumination_temporal_metadata(
        &self,
    ) -> Option<&CompiledHistoryTextureSource> {
        self.global_illumination_temporal_metadata.as_ref()
    }

    pub(crate) const fn screen_space_reflection(&self) -> Option<&CompiledHistoryTextureSource> {
        self.screen_space_reflection.as_ref()
    }

    pub(crate) const fn hzb_furthest(&self) -> Option<&CompiledHistoryTextureSource> {
        self.hzb_furthest.as_ref()
    }

    pub(crate) const fn volumetric_scattering(&self) -> Option<&CompiledHistoryTextureSource> {
        self.volumetric_scattering.as_ref()
    }
}

fn history_texture_source(
    graph: &CompiledRenderGraph,
    binding: &CompiledRenderGraphAccessAllocationBinding,
) -> Result<CompiledHistoryTextureSource, String> {
    let declaration = graph
        .resource_declaration(binding.key.resource)
        .ok_or_else(|| {
            format!(
                "history epilogue access {:?} references an undeclared graph resource",
                binding.key.access_id
            )
        })?;
    if declaration.kind != RenderGraphResourceKind::TransientTexture {
        return Err(format!(
            "history epilogue source `{}` must be a graph-owned texture, got {:?}",
            declaration.name, declaration.kind
        ));
    }
    let RenderGraphResourceDesc::Texture(desc) = &declaration.desc else {
        return Err(format!(
            "history epilogue source `{}` has no texture descriptor",
            declaration.name
        ));
    };
    if !desc.usage.contains(TextureUsage::COPY_SRC) {
        return Err(format!(
            "history epilogue source `{}` must declare COPY_SRC usage",
            declaration.name
        ));
    }
    Ok(CompiledHistoryTextureSource {
        access_id: binding.key.access_id,
        desc: desc.clone(),
    })
}

#[cfg(test)]
mod tests {
    use crate::render_graph::{QueueLane, RenderGraphBuilder};
    use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

    use super::*;

    #[test]
    fn plan_keeps_the_final_live_writer_access_for_history_outputs() {
        let mut builder = RenderGraphBuilder::new("history-epilogue-final-writer");
        let output = builder.create_texture(TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ));
        builder.mark_persistent(output).unwrap();
        let first = builder.add_pass("history-first-writer", QueueLane::Graphics);
        let final_writer = builder.add_pass("history-final-writer", QueueLane::Graphics);
        builder.write_texture(first, output).unwrap();
        builder.write_texture(final_writer, output).unwrap();
        builder.add_dependency(first, final_writer).unwrap();
        let graph = builder.compile().unwrap();
        let final_access = graph.access_id_at(final_writer, 0).unwrap();

        let plan = CompiledHistoryEpiloguePlan::from_graph(&graph).unwrap();

        assert_eq!(
            plan.screen_space_reflection()
                .map(|source| source.access_id()),
            Some(final_access)
        );
    }

    #[test]
    fn plan_rejects_a_history_output_without_copy_source_usage() {
        let mut builder = RenderGraphBuilder::new("history-epilogue-copy-source-contract");
        let output = builder.create_texture(TextureDesc::new(
            PostProcessGraphResourceNames::GLOBAL_ILLUMINATION,
            32,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT,
        ));
        builder.mark_persistent(output).unwrap();
        let writer = builder.add_pass("history-writer", QueueLane::Graphics);
        builder.write_texture(writer, output).unwrap();
        let graph = builder.compile().unwrap();

        let error = CompiledHistoryEpiloguePlan::from_graph(&graph)
            .expect_err("history output must be copyable before frame execution");

        assert!(error.contains("must declare COPY_SRC usage"));
    }
}
