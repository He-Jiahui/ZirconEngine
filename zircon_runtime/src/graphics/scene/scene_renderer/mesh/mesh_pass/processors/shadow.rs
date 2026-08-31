use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassProcessor, shadow_command_spec,
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
        let Some(spec) = shadow_command_spec(batch) else {
            return;
        };
        let pipeline_key = batch.effective_shadow_pipeline_key();
        let pipeline_variant_id =
            context.pipeline_variant_id_with_key(spec.pipeline_kind, batch, &pipeline_key);
        out.push(batch.command_with_pipeline_key(
            spec.phase,
            spec.pipeline_kind,
            pipeline_variant_id,
            &pipeline_key,
        ));
    }
}
