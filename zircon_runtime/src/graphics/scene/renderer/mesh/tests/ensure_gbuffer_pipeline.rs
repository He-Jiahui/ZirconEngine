use crate::core::framework::render::ShaderPassType;
use crate::graphics::scene::resources::default_pipeline_key;

use super::super::mesh_pipeline_deferred_gbuffer_template_source_for_geometry;
use super::{gbuffer_mesh_shader_key, GBUFFER_MESH_SHADER_KEY_PREFIX};

#[test]
fn gbuffer_mesh_shader_key_includes_shader_variant_identity_and_source_hash() {
    let variant_key =
        default_pipeline_key().shader_variant_key(ShaderPassType::GBuffer, "wgpu-runtime");
    let source = match mesh_pipeline_deferred_gbuffer_template_source_for_geometry(
        &default_pipeline_key(),
        variant_key.geometry_source,
    ) {
        Ok(source) => source,
        Err(error) => panic!("deferred gbuffer template source assembly failed: {error:?}"),
    };
    let key = gbuffer_mesh_shader_key(&variant_key, &source.source_hash);

    assert!(key.starts_with(GBUFFER_MESH_SHADER_KEY_PREFIX));
    assert!(key.contains(&variant_key.canonical_string()));
    assert!(key.contains(&source.source_hash));
}
