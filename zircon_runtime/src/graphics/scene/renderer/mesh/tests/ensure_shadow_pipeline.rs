use crate::core::framework::render::ShaderPassType;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;

use super::super::mesh_pipeline_shadow_template_source_for_geometry;
use super::super::PipelineCreationTarget;
use super::{shadow_mesh_shader_key, shadow_pipeline_target, SHADOW_MESH_SHADER_KEY_PREFIX};

#[test]
fn shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash() {
    let variant_key =
        default_pipeline_key().shader_variant_key(ShaderPassType::Shadow, "wgpu-runtime");
    let source = match mesh_pipeline_shadow_template_source_for_geometry(
        &default_pipeline_key(),
        variant_key.geometry_source,
    ) {
        Ok(source) => source,
        Err(error) => panic!("shadow template source assembly failed: {error:?}"),
    };
    let key = shadow_mesh_shader_key(&variant_key, &source.source_hash);

    assert!(key.starts_with(SHADOW_MESH_SHADER_KEY_PREFIX));
    assert!(key.contains(&variant_key.canonical_string()));
    assert!(key.contains(&source.source_hash));
}

#[test]
fn shadow_pipeline_target_preserves_alpha_mask_identity() {
    assert_eq!(
        shadow_pipeline_target(MeshPassPipelineKind::ShadowDepth),
        Some(PipelineCreationTarget::MeshPass(
            MeshPassPipelineKind::ShadowDepth
        ))
    );
    assert_eq!(
        shadow_pipeline_target(MeshPassPipelineKind::ShadowDepthAlphaMask),
        Some(PipelineCreationTarget::MeshPass(
            MeshPassPipelineKind::ShadowDepthAlphaMask
        ))
    );
    assert_eq!(shadow_pipeline_target(MeshPassPipelineKind::Base), None);
}
