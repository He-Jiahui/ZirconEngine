use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassPipelineKind,
    MeshPassProcessor,
};

pub(crate) struct OpaqueBasePassProcessor;

impl MeshPassProcessor for OpaqueBasePassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        if batch.disabled_passes.disables_base() {
            return;
        }
        let phase = match batch.phase() {
            MeshDrawQueuePhase::Opaque => RenderPhase::Opaque3d,
            MeshDrawQueuePhase::AlphaMask => RenderPhase::AlphaMask3d,
            MeshDrawQueuePhase::Transparent => return,
        };
        if !batch.relevant_to_main_phase(phase) {
            return;
        }
        let pipeline_kind = MeshPassPipelineKind::Base;
        let pipeline_variant_id = context.pipeline_variant_id(pipeline_kind, batch);
        out.push(batch.command(phase, pipeline_kind, pipeline_variant_id));
    }
}
