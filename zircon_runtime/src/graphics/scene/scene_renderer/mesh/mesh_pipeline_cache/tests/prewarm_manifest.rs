use crate::core::framework::render::{
    ShaderPassType, ShaderPipelinePrewarmState, ShaderVariantPrewarmRequest,
};
use crate::graphics::scene::resources::default_pipeline_key;

use super::pipeline_key_from_prewarm_request;

#[test]
fn runtime_prewarm_pipeline_state_reconstructs_complete_pipeline_key() {
    let mut expected = default_pipeline_key();
    expected.shader_revision = 19;
    expected.material_layout_hash = 23;
    expected.material_option_bits = 29;
    expected.double_sided = true;
    expected.alpha_blend = true;
    expected.alpha_mask = true;
    expected.alpha_cutoff_bits = Some(0.42_f32.to_bits());
    expected.receive_shadows = false;
    expected.unlit = true;
    expected.has_normal_texture = true;
    expected.pbr_clearcoat = true;
    expected.pbr_anisotropy = true;
    expected.pbr_transmission = true;
    expected.volumetric_fog = true;
    let request = ShaderVariantPrewarmRequest {
        key: expected.shader_variant_key(ShaderPassType::Forward, "wgpu-runtime"),
        pipeline_state: Some(ShaderPipelinePrewarmState {
            alpha_blend: expected.alpha_blend,
            alpha_cutoff_bits: expected.alpha_cutoff_bits,
            unlit: expected.unlit,
        }),
        // Source-table resolution is outside this key-projection unit test.
        source_id: Default::default(),
    };

    assert_eq!(pipeline_key_from_prewarm_request(&request), Ok(expected));
}

#[test]
fn runtime_prewarm_rejects_manifest_request_without_exact_pipeline_state() {
    let key = default_pipeline_key();
    let request = ShaderVariantPrewarmRequest {
        key: key.shader_variant_key(ShaderPassType::Forward, "wgpu-runtime"),
        pipeline_state: None,
        // Source-table resolution is outside this key-projection unit test.
        source_id: Default::default(),
    };

    assert_eq!(
        pipeline_key_from_prewarm_request(&request),
        Err("runtime shader pipeline prewarm requires the exact pipeline_state descriptor")
    );
}
