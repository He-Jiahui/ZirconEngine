use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassPipelineKind,
    MeshPassProcessor,
};

pub(crate) struct VelocityPassProcessor;

impl MeshPassProcessor for VelocityPassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        if batch.queue_profile.early_z_eligible()
            && batch.queue_profile.motion_vector_history_eligible()
            && batch.has_previous_motion_vector_transform
            && batch.relevant_to_main_phase(RenderPhase::PostProcess)
        {
            let pipeline_kind = MeshPassPipelineKind::MotionVector;
            let pipeline_variant_id =
                context.pipeline_variant_id(pipeline_kind, &batch.pipeline_key);
            out.push(batch.command(RenderPhase::PostProcess, pipeline_kind, pipeline_variant_id));
        }
    }
}
