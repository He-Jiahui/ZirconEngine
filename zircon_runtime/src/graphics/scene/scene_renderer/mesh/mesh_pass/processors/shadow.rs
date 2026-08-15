use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    shadow_command_spec, MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext, MeshPassProcessor,
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
        let pipeline_variant_id = context.pipeline_variant_id(spec.pipeline_kind, batch);
        out.push(batch.command(spec.phase, spec.pipeline_kind, pipeline_variant_id));
    }
}
