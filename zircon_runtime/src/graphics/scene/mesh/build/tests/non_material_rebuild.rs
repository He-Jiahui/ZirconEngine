use crate::core::framework::render::{RenderPhase, RenderPhaseSortComponents};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshBatchRef, MeshDrawArgs, MeshGeometryHandle, MeshPassPipelineKind,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantRegistry;

use super::{can_rebuild_non_material_command_phase, rebuild_non_material_command};

#[test]
fn rebuilds_opaque_shadow_command_without_material_handles() {
    let batch = batch(MeshDrawQueuePhase::Opaque, true);
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut context =
        crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassBuildContext::with_default_quality(
            &mut variants,
        );

    let shadow = rebuild_non_material_command(&batch, RenderPhase::Shadow, &mut context)
        .expect("shadow command should rebuild without material bind groups");

    assert_eq!(shadow.pipeline_kind, MeshPassPipelineKind::ShadowDepth);
    assert_ne!(
        shadow.pipeline_variant_id,
        crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPipelineVariantId::new(0)
    );
    assert!(shadow.material.is_none());
    assert!(shadow.standard_material.is_none());
}

#[test]
fn alpha_mask_shadow_is_not_pre_mesh_draw_rebuildable() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut context =
        crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassBuildContext::with_default_quality(
            &mut variants,
        );
    assert!(rebuild_non_material_command(
        &batch(MeshDrawQueuePhase::AlphaMask, true),
        RenderPhase::Shadow,
        &mut context,
    )
    .is_none());
}

#[test]
fn depth_and_material_phases_are_not_pre_mesh_draw_rebuildable() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut context =
        crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassBuildContext::with_default_quality(
            &mut variants,
        );
    assert!(!can_rebuild_non_material_command_phase(
        RenderPhase::Prepass
    ));
    assert!(!can_rebuild_non_material_command_phase(
        RenderPhase::Opaque3d
    ));
    assert!(rebuild_non_material_command(
        &batch(MeshDrawQueuePhase::Opaque, true),
        RenderPhase::Prepass,
        &mut context,
    )
    .is_none());
    assert!(rebuild_non_material_command(
        &batch(MeshDrawQueuePhase::Opaque, true),
        RenderPhase::Opaque3d,
        &mut context,
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
    .with_cache_identity(7, (7 << 16) | 1, 1)
    .with_casts_shadow(casts_shadow)
    .with_gpu_scene_instance_span(4, 2)
}
