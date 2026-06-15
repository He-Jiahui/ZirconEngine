use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassPipelineKind,
    MeshPassProcessor,
};

pub(crate) struct TaaReactiveMaskPassProcessor;

impl MeshPassProcessor for TaaReactiveMaskPassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        let Some(pipeline_kind) = reactive_mask_pipeline_kind(batch) else {
            return;
        };
        let pipeline_variant_id = context.pipeline_variant_id(pipeline_kind, &batch.pipeline_key);
        out.push(batch.command(RenderPhase::PostProcess, pipeline_kind, pipeline_variant_id));
    }
}

fn reactive_mask_pipeline_kind(batch: &MeshBatchRef) -> Option<MeshPassPipelineKind> {
    match batch.phase() {
        MeshDrawQueuePhase::Transparent
            if batch.relevant_to_main_phase(RenderPhase::Transparent3d) =>
        {
            Some(MeshPassPipelineKind::TaaReactiveMask)
        }
        MeshDrawQueuePhase::Opaque
            if batch.has_taa_reactive_material_mask()
                && batch.relevant_to_main_phase(RenderPhase::Opaque3d) =>
        {
            Some(MeshPassPipelineKind::TaaReactiveMaterialMask)
        }
        MeshDrawQueuePhase::AlphaMask
            if batch.has_taa_reactive_material_mask()
                && batch.relevant_to_main_phase(RenderPhase::AlphaMask3d) =>
        {
            Some(MeshPassPipelineKind::TaaReactiveMaterialMask)
        }
        _ => None,
    }
}
