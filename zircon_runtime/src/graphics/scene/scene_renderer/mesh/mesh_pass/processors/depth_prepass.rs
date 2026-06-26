use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassPipelineKind,
    MeshPassProcessor,
};

pub(crate) struct DepthPrepassProcessor;

impl MeshPassProcessor for DepthPrepassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        if batch.queue_profile.early_z_eligible()
            && batch.relevant_to_main_phase(RenderPhase::Prepass)
        {
            let pipeline_kind = MeshPassPipelineKind::DepthPrepass;
            let pipeline_variant_id = context.pipeline_variant_id(pipeline_kind, batch);
            out.push(batch.command(RenderPhase::Prepass, pipeline_kind, pipeline_variant_id));
        }
    }
}
