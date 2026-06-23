use crate::core::framework::render::RenderPhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshBatchRef, MeshDrawCommand, MeshPassPipelineKind, MeshPipelineVariantId,
};

pub(super) fn can_rebuild_non_material_command_phase(phase: RenderPhase) -> bool {
    phase == RenderPhase::Shadow
}

pub(super) fn rebuild_non_material_command(
    batch: &MeshBatchRef,
    phase: RenderPhase,
) -> Option<MeshDrawCommand> {
    match phase {
        RenderPhase::Shadow if batch.casts_shadow && batch.relevant_to_shadow_view() => {
            let pipeline_kind = match batch.phase() {
                MeshDrawQueuePhase::Opaque => MeshPassPipelineKind::ShadowDepth,
                MeshDrawQueuePhase::AlphaMask | MeshDrawQueuePhase::Transparent => return None,
            };
            Some(batch.command(
                RenderPhase::Shadow,
                pipeline_kind,
                MeshPipelineVariantId::new(0),
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{RenderPhase, RenderPhaseSortComponents};
    use crate::core::framework::scene::Mobility;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
        MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshBatchRef, MeshDrawArgs, MeshGeometryHandle, MeshPassPipelineKind,
    };

    use super::{can_rebuild_non_material_command_phase, rebuild_non_material_command};

    #[test]
    fn rebuilds_opaque_shadow_command_without_material_handles() {
        let batch = batch(MeshDrawQueuePhase::Opaque, true);

        let shadow = rebuild_non_material_command(&batch, RenderPhase::Shadow)
            .expect("shadow command should rebuild without material bind groups");

        assert_eq!(shadow.pipeline_kind, MeshPassPipelineKind::ShadowDepth);
        assert!(shadow.material.is_none());
        assert!(shadow.standard_material.is_none());
    }

    #[test]
    fn alpha_mask_shadow_is_not_pre_mesh_draw_rebuildable() {
        assert!(rebuild_non_material_command(
            &batch(MeshDrawQueuePhase::AlphaMask, true),
            RenderPhase::Shadow,
        )
        .is_none());
    }

    #[test]
    fn depth_and_material_phases_are_not_pre_mesh_draw_rebuildable() {
        assert!(!can_rebuild_non_material_command_phase(
            RenderPhase::Prepass
        ));
        assert!(!can_rebuild_non_material_command_phase(
            RenderPhase::Opaque3d
        ));
        assert!(rebuild_non_material_command(
            &batch(MeshDrawQueuePhase::Opaque, true),
            RenderPhase::Prepass,
        )
        .is_none());
        assert!(rebuild_non_material_command(
            &batch(MeshDrawQueuePhase::Opaque, true),
            RenderPhase::Opaque3d,
        )
        .is_none());
    }

    fn batch(phase: MeshDrawQueuePhase, casts_shadow: bool) -> MeshBatchRef {
        MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                phase,
                MeshDrawGeometrySource::Prepared,
                Mobility::Static,
                false,
                false,
                false,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(0.0, 1),
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_cache_identity(7, 1)
        .with_casts_shadow(casts_shadow)
        .with_gpu_scene_instance_span(4, 2)
    }
}
