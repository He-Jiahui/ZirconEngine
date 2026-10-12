use crate::core::framework::render::ShaderPassType;
use crate::graphics::scene::resources::default_pipeline_key;

use super::{hit_proxy_mesh_shader_key, HIT_PROXY_MESH_SHADER_KEY_PREFIX};

#[test]
fn hit_proxy_shader_key_keeps_pass_identity_and_source_hash() {
    let variant_key =
        default_pipeline_key().shader_variant_key(ShaderPassType::HitProxy, "wgpu-runtime");
    let key = hit_proxy_mesh_shader_key(&variant_key, "source-a");

    assert!(key.starts_with(HIT_PROXY_MESH_SHADER_KEY_PREFIX));
    assert!(key.contains("|pass=hit_proxy|"));
    assert!(key.ends_with("#source-a"));
}
