use crate::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::resource::{ResourceId, ResourceKind, ResourceManager, ResourceRecord};
use crate::graphics::scene::resources::default_pipeline_key;

fn shader_publication() -> crate::core::resource::ResourceReadinessRowIdentity {
    let resources = ResourceManager::new();
    let locator = super::fallback_shader_uri();
    let id = ResourceId::from_locator(&locator);
    resources
        .register_record(ResourceRecord::new(id, ResourceKind::Shader, locator))
        .unwrap();
    resources.readiness_generation().row_identity(id).unwrap()
}

#[test]
fn pipeline_key_identifies_builtin_fallback_shader() {
    let mut key = default_pipeline_key();
    assert!(key.uses_fallback_shader());

    key.shader_id = ResourceId::from_stable_label("res://shaders/custom-material.wgsl");
    assert!(!key.uses_fallback_shader());
}

#[test]
fn pipeline_key_derives_material_shader_variant_key() {
    let mut key = default_pipeline_key();
    key.shader_revision = 42;
    key.shader_dependency_identity = Some(shader_publication());
    key.double_sided = true;
    key.alpha_mask = true;
    key.receive_shadows = true;

    let variant = key.shader_variant_key(ShaderPassType::GBuffer, "wgpu-test");

    assert_eq!(variant.material_shader, key.shader_id);
    assert_eq!(variant.material_revision, 42);
    assert!(!variant.canonical_string().contains("dependency_revision"));
    assert!(!variant.canonical_string().contains("dependency_identity"));
    assert_eq!(variant.shading_model, SHADING_MODEL_ID_STANDARD_PBR);
    assert_eq!(variant.pass_type, ShaderPassType::GBuffer);
    assert_eq!(variant.platform_token, "wgpu-test");
    assert!(variant.features.contains(ShaderFeatureBits::ALPHA_TEST));
    assert!(variant.features.contains(ShaderFeatureBits::DOUBLE_SIDED));
    assert!(variant
        .features
        .contains(ShaderFeatureBits::RECEIVE_SHADOWS));
    assert!(!variant
        .features
        .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE));
}

#[test]
fn pipeline_key_separates_runtime_dependency_generations_without_changing_disk_variant_key() {
    let mut first = default_pipeline_key();
    first.shader_dependency_identity = Some(shader_publication());
    let mut second = first.clone();
    assert_eq!(first, second);
    second.shader_dependency_identity = Some(shader_publication());

    assert_ne!(first, second);
    assert_eq!(
        first.shader_variant_key(ShaderPassType::Forward, "wgpu-test"),
        second.shader_variant_key(ShaderPassType::Forward, "wgpu-test"),
        "retained process-local publications must not enter the persistent variant contract"
    );
}

#[test]
fn reverse_raster_winding_changes_pso_identity_without_a_shader_permutation() {
    let first = default_pipeline_key();
    let mut mirrored = first.clone();
    mirrored.reverse_raster_winding = true;

    assert_ne!(
        first.pipeline_variant_identity(),
        mirrored.pipeline_variant_identity()
    );
    assert_eq!(
        first.shader_variant_key(ShaderPassType::Forward, "wgpu-test"),
        mirrored.shader_variant_key(ShaderPassType::Forward, "wgpu-test"),
        "raster winding must not duplicate persistent WGSL variants"
    );
}

#[test]
fn pipeline_key_derives_normal_texture_shader_feature() {
    let mut key = default_pipeline_key();
    key.has_normal_texture = true;

    let variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

    assert!(variant
        .features
        .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE));
}

#[test]
fn pipeline_key_can_disable_receive_shadow_shader_feature() {
    let mut key = default_pipeline_key();
    key.receive_shadows = false;

    let variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

    assert!(!variant
        .features
        .contains(ShaderFeatureBits::RECEIVE_SHADOWS));
}

#[test]
fn render_advanced_material_pipeline_key_tracks_authored_lobes() {
    let mut key = default_pipeline_key();

    let default_variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");
    assert!(!default_variant
        .features
        .contains(ShaderFeatureBits::PBR_CLEARCOAT));
    assert!(!default_variant
        .features
        .contains(ShaderFeatureBits::PBR_ANISOTROPY));
    assert!(!default_variant
        .features
        .contains(ShaderFeatureBits::PBR_TRANSMISSION));

    key.pbr_clearcoat = true;
    key.pbr_anisotropy = true;
    key.pbr_ior_override = true;
    key.pbr_transmission = true;
    let advanced_variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

    assert!(advanced_variant
        .features
        .contains(ShaderFeatureBits::PBR_CLEARCOAT));
    assert!(advanced_variant
        .features
        .contains(ShaderFeatureBits::PBR_ANISOTROPY));
    assert!(advanced_variant
        .features
        .contains(ShaderFeatureBits::PBR_TRANSMISSION));
    assert!(key.requires_forward_path());
}

#[test]
fn pipeline_key_routes_non_default_ior_without_a_shader_permutation() {
    let mut key = default_pipeline_key();
    let baseline = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

    key.pbr_ior_override = true;
    let routed = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

    assert!(key.requires_forward_path());
    assert_eq!(routed, baseline);
    assert_eq!(key.pipeline_variant_identity(), default_pipeline_key());
}
