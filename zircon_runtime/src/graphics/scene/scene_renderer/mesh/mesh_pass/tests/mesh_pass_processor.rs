use crate::core::framework::render::{
    GeometrySourceId, RenderPhase, RenderPhaseSortComponents, ShaderQualityTier,
    GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::{default_pipeline_key, PipelineKey};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::{
    MeshBatchRef, MeshDrawArgs, MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};

#[test]
fn mesh_batch_ref_attaches_previous_geometry_only_to_velocity_commands() {
    let batch = MeshBatchRef::new(
        MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::DynamicGpuSkinningSource,
            Mobility::Dynamic,
            true,
            true,
            true,
        ),
        default_pipeline_key(),
        RenderPhaseSortComponents::new(0.0, 10),
        MeshGeometryHandle::test(1),
        MeshDrawArgs::direct_indexed(0, 3),
    )
    .with_gpu_scene_instance_span(0, 1)
    .with_previous_velocity_geometry(MeshGeometryHandle::test(2));

    let velocity = batch.command(
        RenderPhase::PostProcess,
        MeshPassPipelineKind::Velocity,
        MeshPipelineVariantId::new(1),
    );
    let base = batch.command(
        RenderPhase::Opaque3d,
        MeshPassPipelineKind::Base,
        MeshPipelineVariantId::new(2),
    );

    assert_eq!(
        velocity
            .previous_velocity_geometry
            .as_ref()
            .map(MeshGeometryHandle::id),
        Some(2)
    );
    assert!(base.previous_velocity_geometry.is_none());
}

#[test]
fn mesh_pass_build_context_resolves_prepared_gpu_skinning_as_skinned_variant() {
    let mut resolver = CapturingVariantResolver::default();
    let mut context = super::MeshPassBuildContext::new(&mut resolver, ShaderQualityTier::Medium);
    let batch = MeshBatchRef::new(
        MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::Prepared,
            Mobility::Dynamic,
            false,
            true,
            false,
        ),
        default_pipeline_key(),
        RenderPhaseSortComponents::new(0.0, 10),
        MeshGeometryHandle::test(1),
        MeshDrawArgs::direct_indexed(0, 3),
    );

    let variant_id = context.pipeline_variant_id(MeshPassPipelineKind::Base, &batch);

    assert_eq!(variant_id, MeshPipelineVariantId::new(9));
    assert_eq!(
        resolver.last_geometry_source,
        Some(GEOMETRY_SOURCE_ID_SKINNED_MESH)
    );
}

#[test]
fn mesh_pass_build_context_resolves_cpu_morphed_gpu_skinning_as_skinned_variant() {
    let mut resolver = CapturingVariantResolver::default();
    let mut context = super::MeshPassBuildContext::new(&mut resolver, ShaderQualityTier::Medium);
    let batch = MeshBatchRef::new(
        MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource,
            Mobility::Dynamic,
            false,
            true,
            false,
        ),
        default_pipeline_key(),
        RenderPhaseSortComponents::new(0.0, 10),
        MeshGeometryHandle::test(1),
        MeshDrawArgs::direct_indexed(0, 3),
    );

    let variant_id = context.pipeline_variant_id(MeshPassPipelineKind::Base, &batch);

    assert_eq!(variant_id, MeshPipelineVariantId::new(9));
    assert_eq!(
        resolver.last_geometry_source,
        Some(GEOMETRY_SOURCE_ID_SKINNED_MESH)
    );
}

#[test]
fn mesh_pass_build_context_resolves_gpu_skinned_morphed_as_skinned_morphed_variant() {
    let mut resolver = CapturingVariantResolver::default();
    let mut context = super::MeshPassBuildContext::new(&mut resolver, ShaderQualityTier::Medium);
    let batch = MeshBatchRef::new(
        MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource,
            Mobility::Dynamic,
            false,
            true,
            false,
        ),
        default_pipeline_key(),
        RenderPhaseSortComponents::new(0.0, 10),
        MeshGeometryHandle::test(1),
        MeshDrawArgs::direct_indexed(0, 3),
    );

    let variant_id = context.pipeline_variant_id(MeshPassPipelineKind::Base, &batch);

    assert_eq!(variant_id, MeshPipelineVariantId::new(9));
    assert_eq!(
        resolver.last_geometry_source,
        Some(GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH)
    );
}

#[derive(Default)]
struct CapturingVariantResolver {
    last_geometry_source: Option<GeometrySourceId>,
}

impl MeshPipelineVariantResolver for CapturingVariantResolver {
    fn resolve_variant_for_geometry(
        &mut self,
        _kind: MeshPassPipelineKind,
        _pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        _shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        self.last_geometry_source = Some(geometry_source);
        MeshPipelineVariantId::new(9)
    }
}
