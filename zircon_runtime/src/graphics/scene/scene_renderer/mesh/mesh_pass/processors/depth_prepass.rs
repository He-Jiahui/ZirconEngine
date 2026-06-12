use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassPipelineKind,
    MeshPassProcessor, MeshPipelineVariantId,
};

pub(crate) struct DepthPrepassProcessor;

impl MeshPassProcessor for DepthPrepassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        _context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        if batch.queue_profile.early_z_eligible()
            && batch.relevant_to_main_phase(RenderPhase::Prepass)
        {
            out.push(batch.command(
                RenderPhase::Prepass,
                MeshPassPipelineKind::DepthPrepass,
                MeshPipelineVariantId::new(0),
            ));
        }
    }
}
