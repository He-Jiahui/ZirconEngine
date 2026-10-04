use crate::core::framework::render::ShaderPassType;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;

use super::super::mesh_pipeline_taa_reactive_mask_template_source_for_geometry;
use super::super::PipelineCreationTarget;
use super::{
    taa_reactive_mask_mesh_shader_key, taa_reactive_pipeline_target,
    TAA_REACTIVE_MASK_MESH_SHADER_KEY_PREFIX,
};

#[test]
fn taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash() {
    let variant_key =
        default_pipeline_key().shader_variant_key(ShaderPassType::TaaReactiveMask, "wgpu-runtime");
    let source = match mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
        &default_pipeline_key(),
        variant_key.geometry_source,
    ) {
        Ok(source) => source,
        Err(error) => panic!("TAA reactive mask template source assembly failed: {error:?}"),
    };
    let key = taa_reactive_mask_mesh_shader_key(&variant_key, &source.source_hash);

    assert!(key.starts_with(TAA_REACTIVE_MASK_MESH_SHADER_KEY_PREFIX));
    assert!(key.contains(&variant_key.canonical_string()));
    assert!(key.contains("|pass=taa_reactive_mask|"));
    assert!(key.contains(&source.source_hash));
}

#[test]
fn taa_reactive_pipeline_target_preserves_material_mask_identity() {
    assert_eq!(
        taa_reactive_pipeline_target(MeshPassPipelineKind::TaaReactiveMask),
        Some(PipelineCreationTarget::MeshPass(
            MeshPassPipelineKind::TaaReactiveMask
        ))
    );
    assert_eq!(
        taa_reactive_pipeline_target(MeshPassPipelineKind::TaaReactiveMaterialMask),
        Some(PipelineCreationTarget::MeshPass(
            MeshPassPipelineKind::TaaReactiveMaterialMask
        ))
    );
    assert_eq!(
        taa_reactive_pipeline_target(MeshPassPipelineKind::Base),
        None
    );
}
