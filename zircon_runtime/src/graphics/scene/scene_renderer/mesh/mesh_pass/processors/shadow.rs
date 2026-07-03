use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassPipelineKind,
    MeshPassProcessor,
};

pub(crate) struct ShadowPassProcessor;

impl MeshPassProcessor for ShadowPassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        if batch.disabled_passes.disables_shadow() {
            return;
        }
        if !batch.casts_shadow || !batch.relevant_to_shadow_view() {
            return;
        }
        let pipeline_kind = match batch.phase() {
            MeshDrawQueuePhase::AlphaMask => MeshPassPipelineKind::ShadowDepthAlphaMask,
            MeshDrawQueuePhase::Opaque => MeshPassPipelineKind::ShadowDepth,
            MeshDrawQueuePhase::Transparent => return,
        };
        let pipeline_variant_id = context.pipeline_variant_id(pipeline_kind, batch);
        out.push(batch.command(RenderPhase::Shadow, pipeline_kind, pipeline_variant_id));
    }
}
