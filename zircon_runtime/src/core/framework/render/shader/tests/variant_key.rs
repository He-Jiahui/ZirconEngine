use zircon_runtime_interface::resource::ResourceId;

use crate::core::framework::render::{
    GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    ShadingModelId, SHADING_MODEL_ID_BLINN_PHONG,
};

#[test]
fn render_shader_variant_key_packs_dimensions_stably() {
    let key = ShaderVariantKey {
        material_shader: ResourceId::from_stable_label("builtin://pbr"),
        material_revision: 7,
        material_layout_hash: 0,
        material_option_bits: 0x13,
        geometry_source: GeometrySourceId::new(3),
        shading_model: SHADING_MODEL_ID_BLINN_PHONG,
        pass_type: ShaderPassType::Velocity,
        features: ShaderFeatureBits::new(
            ShaderFeatureBits::ALPHA_TEST | ShaderFeatureBits::DOUBLE_SIDED,
        ),
        quality: ShaderQualityTier::High,
        platform_token: "wgpu-vulkan-downlevel-default".to_string(),
    };

    assert_eq!(
        key.packed_dims(),
        3 | (1 << 8) | (4 << 16) | (0b101 << 20) | (2 << 52)
    );
    assert_eq!(
        key.canonical_string(),
        format!(
            concat!(
                "shader_variant_v1",
                "|material={}",
                "|revision=7",
                "|layout=0x0000000000000000",
                "|material_options=0x00000013",
                "|geometry=3",
                "|shading=1",
                "|pass=velocity",
                "|features=0x00000005",
                "|quality=high",
                "|platform=wgpu-vulkan-downlevel-default"
            ),
            ResourceId::from_stable_label("builtin://pbr")
        )
    );
}

#[test]
fn render_shader_variant_key_keeps_plugin_geometry_and_shading_fields_disjoint() {
    let plugin_geometry = ShaderVariantKey {
        material_shader: ResourceId::from_stable_label("builtin://pbr"),
        material_revision: 1,
        material_layout_hash: 0,
        material_option_bits: 0,
        geometry_source: GeometrySourceId::new(16),
        shading_model: ShadingModelId::new(0),
        pass_type: ShaderPassType::Forward,
        features: ShaderFeatureBits::default(),
        quality: ShaderQualityTier::Low,
        platform_token: "test".to_string(),
    };
    let mut first_plugin_shading_model = plugin_geometry.clone();
    first_plugin_shading_model.geometry_source = GeometrySourceId::new(0);
    first_plugin_shading_model.shading_model = ShadingModelId::new(1);

    assert_eq!(plugin_geometry.packed_dims(), 16);
    assert_eq!(first_plugin_shading_model.packed_dims(), 1 << 8);
    assert_ne!(
        plugin_geometry.packed_dims(),
        first_plugin_shading_model.packed_dims()
    );
}

#[test]
fn render_shader_feature_bits_reports_named_flags() {
    let features = ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST).union(
        ShaderFeatureBits::new(ShaderFeatureBits::INSTANCED_PREV_TRANSFORM),
    );

    assert!(features.contains(ShaderFeatureBits::ALPHA_TEST));
    assert!(features.contains(ShaderFeatureBits::INSTANCED_PREV_TRANSFORM));
    assert!(!features.contains(ShaderFeatureBits::RECEIVE_SHADOWS));
    assert!(!features.contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));
}

#[test]
fn render_shader_pass_type_names_taa_reactive_mask_separately_from_forward() {
    assert_eq!(ShaderPassType::Forward.packed_value(), 0);
    assert_eq!(ShaderPassType::TaaReactiveMask.packed_value(), 5);
    assert_eq!(ShaderPassType::TaaReactiveMask.token(), "taa_reactive_mask");
}

#[test]
fn render_shader_pass_type_reserves_a_stable_hit_proxy_identity() {
    assert_eq!(ShaderPassType::HitProxy.packed_value(), 6);
    assert_eq!(ShaderPassType::HitProxy.token(), "hit_proxy");
}
