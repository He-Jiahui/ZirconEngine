use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshBatchRef, MeshDrawCommand, MeshPassBuildContext, MeshPassPipelineKind,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

pub(super) fn can_rebuild_non_material_command_phase(phase: RenderPhase) -> bool {
    phase == RenderPhase::Shadow
}

pub(super) fn rebuild_non_material_command<R>(
    batch: &MeshBatchRef,
    phase: RenderPhase,
    context: &mut MeshPassBuildContext<'_, R>,
) -> Option<MeshDrawCommand>
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    match phase {
        RenderPhase::Shadow
            if !batch.disabled_passes.disables_shadow()
                && batch.casts_shadow
                && batch.relevant_to_shadow_view() =>
        {
            let pipeline_kind = match batch.phase() {
                MeshDrawQueuePhase::Opaque => MeshPassPipelineKind::ShadowDepth,
                MeshDrawQueuePhase::AlphaMask | MeshDrawQueuePhase::Transparent => return None,
            };
            let pipeline_variant_id = context.pipeline_variant_id(pipeline_kind, batch);
            Some(batch.command(RenderPhase::Shadow, pipeline_kind, pipeline_variant_id))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/non_material_rebuild.rs"]
mod tests;
