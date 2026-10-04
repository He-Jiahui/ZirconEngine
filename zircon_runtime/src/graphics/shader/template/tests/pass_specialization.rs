use super::*;

fn forward_shadow_include(features: ShaderFeatureBits) -> ShaderTemplateInclude {
    pass_template_for(ShaderPassType::Forward, features)
        .support_includes
        .into_iter()
        .find(|include| include.token == "zr_shadow.wgsl")
        .expect("forward shader template should provide the shadow API")
}

#[test]
fn forward_without_receive_shadows_uses_binding_free_shadow_stub() {
    let include = forward_shadow_include(ShaderFeatureBits::default());

    assert!(include
        .source
        .contains("fn zr_gpu_light_shadow_visibility("));
    assert!(include.source.contains("return 1.0;"));
    assert!(!include.source.contains("@group(1) @binding(8)"));
    assert!(!include.source.contains("fn zr_sample_shadow_slot("));
}

#[test]
fn forward_with_receive_shadows_uses_full_shadow_module() {
    let include =
        forward_shadow_include(ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS));

    assert!(include.source.contains("@group(1) @binding(8)"));
    assert!(include.source.contains("fn zr_sample_shadow_slot("));
}

#[test]
fn shadow_specialization_preserves_token_and_changes_content_hash() {
    let disabled = forward_shadow_include(ShaderFeatureBits::default());
    let enabled =
        forward_shadow_include(ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS));

    assert_eq!(disabled.token, enabled.token);
    assert_ne!(disabled.content_hash, enabled.content_hash);
}

#[test]
fn custom_forward_models_keep_the_full_pbr_support_module_with_environment_only_features() {
    let pbr_extras = pass_template_for_shading_model(
        ShaderPassType::Forward,
        ShaderFeatureBits::new(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
        false,
    );
    assert_eq!(pbr_extras.include.token, FORWARD_TEMPLATE_TOKEN);
    let pbr_extras = pbr_extras
        .support_includes
        .into_iter()
        .find(|include| include.token == "zr_pbr_extras.wgsl")
        .expect("custom Forward template should retain the PBR support module");

    for required in [
        "@group(1) @binding(31) var zr_transmission_scene_color",
        "@group(1) @binding(38) var<uniform> zr_transmission_scene_color_params",
        "fn zr_aniso_ggx(",
        "fn zr_clearcoat_lobe(",
        "fn zr_pbr_screen_space_transmission(",
    ] {
        assert!(
            pbr_extras.source.contains(required),
            "custom Forward PBR support must retain `{required}`"
        );
    }
}
