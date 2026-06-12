use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassPipelineKind,
    MeshPassProcessor, MeshPipelineVariantId,
};

pub(crate) struct ShadowPassProcessor;

impl MeshPassProcessor for ShadowPassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        _context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        if !batch.casts_shadow || !batch.relevant_to_shadow_view() {
            return;
        }
        let pipeline_kind = match batch.phase() {
            MeshDrawQueuePhase::AlphaMask => MeshPassPipelineKind::ShadowDepthAlphaMask,
            MeshDrawQueuePhase::Opaque => MeshPassPipelineKind::ShadowDepth,
            MeshDrawQueuePhase::Transparent => return,
        };
        out.push(batch.command(
            RenderPhase::Shadow,
            pipeline_kind,
            MeshPipelineVariantId::new(0),
        ));
    }
}
