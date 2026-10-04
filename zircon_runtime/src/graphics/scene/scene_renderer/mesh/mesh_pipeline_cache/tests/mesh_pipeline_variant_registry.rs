use crate::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderPipelineTarget, ShaderQualityTier,
    GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::{next_pipeline_variant_id, MeshPipelineVariantRegistry};

#[test]
fn mesh_pipeline_variant_registry_reuses_pass_pipeline_shape_id() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let first =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    let second =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);

    assert_eq!(first, second);
    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry.key_for_variant(first).map(|key| key.kind()),
        Some(MeshPassPipelineKind::Base)
    );
}

#[test]
fn resolver_configuration_epoch_advances_only_when_variant_policy_changes() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();
    let initial_epoch = registry.configuration_epoch();

    let _ = registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    assert_eq!(registry.configuration_epoch(), initial_epoch);

    registry.enable_environment_only_pbr_base_profile();
    let environment_only_epoch = registry.configuration_epoch();
    assert_ne!(environment_only_epoch, initial_epoch);

    registry.enable_environment_only_pbr_base_profile();
    assert_eq!(registry.configuration_epoch(), environment_only_epoch);

    registry.disable_environment_only_pbr_base_profile();
    assert_ne!(registry.configuration_epoch(), environment_only_epoch);
}

#[test]
fn mesh_pipeline_variant_id_allocation_rejects_wrapping() {
    let last_variant_count =
        usize::try_from(u32::MAX - 1).expect("u32 variant count must fit the host address space");
    let exhausted_variant_count =
        usize::try_from(u32::MAX).expect("u32 variant count must fit the host address space");

    assert_eq!(
        next_pipeline_variant_id(last_variant_count).value(),
        u32::MAX
    );
    assert!(
        std::panic::catch_unwind(|| next_pipeline_variant_id(exhausted_variant_count)).is_err()
    );
}

#[test]
fn mesh_pipeline_variant_registry_does_not_duplicate_pso_for_ior_routing() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let baseline = default_pipeline_key();
    let mut routed = baseline.clone();
    routed.pbr_ior_override = true;

    let baseline_id = registry.resolve_variant(
        MeshPassPipelineKind::Base,
        &baseline,
        ShaderQualityTier::Medium,
    );
    let routed_id = registry.resolve_variant(
        MeshPassPipelineKind::Base,
        &routed,
        ShaderQualityTier::Medium,
    );

    assert_eq!(baseline_id, routed_id);
    assert_eq!(registry.len(), 1);
}

#[test]
fn environment_only_profile_excludes_non_default_ior() {
    let mut registry = MeshPipelineVariantRegistry::default();
    registry.enable_environment_only_pbr_base_profile();
    let mut key = default_pipeline_key();
    key.pbr_ior_override = true;

    let variant =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    let variant_key = registry
        .key_for_variant(variant)
        .expect("non-default IOR Base variant key");

    assert!(!variant_key
        .shader_variant_key()
        .features
        .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));
    assert!(registry.base_pipeline_requires_forward_receiver(variant));
}

#[test]
fn environment_only_profile_specializes_only_compatible_base_variants() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let generic =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    assert!(!registry
        .key_for_variant(generic)
        .expect("generic Base variant key")
        .shader_variant_key()
        .features
        .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));
    assert!(registry.base_pipeline_requires_forward_receiver(generic));

    registry.enable_environment_only_pbr_base_profile();
    let specialized =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    assert_ne!(specialized, generic);
    assert!(registry
        .key_for_variant(specialized)
        .expect("environment-only Base variant key")
        .shader_variant_key()
        .features
        .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));
    assert!(!registry.base_pipeline_requires_forward_receiver(specialized));

    let gbuffer = registry.resolve_variant(
        MeshPassPipelineKind::GBuffer,
        &key,
        ShaderQualityTier::Medium,
    );
    assert!(!registry
        .key_for_variant(gbuffer)
        .expect("environment-only GBuffer variant key")
        .shader_variant_key()
        .features
        .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));

    let mut shadow_receiver = key;
    shadow_receiver.receive_shadows = true;
    let shadow_receiver = registry.resolve_variant(
        MeshPassPipelineKind::Base,
        &shadow_receiver,
        ShaderQualityTier::Medium,
    );
    assert!(!registry
        .key_for_variant(shadow_receiver)
        .expect("shadow-receiving Base variant key")
        .shader_variant_key()
        .features
        .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));
}

#[test]
fn environment_only_profile_falls_back_to_generic_after_local_provider_upgrade() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    registry.enable_environment_only_pbr_base_profile();
    let specialized =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    registry.disable_environment_only_pbr_base_profile();
    let generic =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);

    assert_ne!(generic, specialized);
    assert!(registry
        .key_for_variant(specialized)
        .expect("specialized Base variant key")
        .shader_variant_key()
        .features
        .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));
    assert!(!registry
        .key_for_variant(generic)
        .expect("generic Base variant key after provider upgrade")
        .shader_variant_key()
        .features
        .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR));
}

#[test]
fn mesh_pipeline_variant_registry_separates_pass_and_pipeline_shape() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let base_key = default_pipeline_key();
    let mut alpha_key = base_key.clone();
    alpha_key.alpha_mask = true;

    let base = registry.resolve_variant(
        MeshPassPipelineKind::Base,
        &base_key,
        ShaderQualityTier::Medium,
    );
    let alpha = registry.resolve_variant(
        MeshPassPipelineKind::Base,
        &alpha_key,
        ShaderQualityTier::Medium,
    );
    let shadow = registry.resolve_variant(
        MeshPassPipelineKind::ShadowDepth,
        &base_key,
        ShaderQualityTier::Medium,
    );

    assert_ne!(base, MeshPipelineVariantId::new(0));
    assert_ne!(base, alpha);
    assert_ne!(base, shadow);
    assert_ne!(alpha, shadow);
    assert_eq!(registry.len(), 3);
}

#[test]
fn mesh_pipeline_variant_registry_derives_material_shader_variant_key() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let mut key = default_pipeline_key();
    key.alpha_mask = true;

    let variant = registry.resolve_variant_for_geometry(
        MeshPassPipelineKind::ShadowDepthAlphaMask,
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ShaderQualityTier::Medium,
    );
    let variant_key = registry.key_for_variant(variant).unwrap();
    let shader_variant = variant_key.shader_variant_key();

    assert_eq!(shader_variant.pass_type, ShaderPassType::Shadow);
    assert_eq!(
        shader_variant.geometry_source,
        GEOMETRY_SOURCE_ID_SKINNED_MESH
    );
    assert_eq!(shader_variant.shading_model, SHADING_MODEL_ID_STANDARD_PBR);
    assert!(shader_variant
        .features
        .contains(ShaderFeatureBits::ALPHA_TEST));
    assert!(shader_variant
        .canonical_string()
        .contains("|platform=wgpu-runtime"));
}

#[test]
fn mesh_pipeline_variant_registry_separates_shader_quality_tiers() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let medium =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    let high = registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::High);

    assert_ne!(medium, high);
    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry
            .key_for_variant(high)
            .map(|key| key.shader_variant_key().quality),
        Some(ShaderQualityTier::High)
    );
}

#[test]
fn mesh_pipeline_variant_registry_separates_geometry_sources() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let static_variant = registry.resolve_variant_for_geometry(
        MeshPassPipelineKind::Base,
        &key,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        ShaderQualityTier::Medium,
    );
    let skinned_variant = registry.resolve_variant_for_geometry(
        MeshPassPipelineKind::Base,
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ShaderQualityTier::Medium,
    );

    assert_ne!(static_variant, skinned_variant);
    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry
            .key_for_variant(skinned_variant)
            .map(|key| key.shader_variant_key().geometry_source),
        Some(GEOMETRY_SOURCE_ID_SKINNED_MESH)
    );
}

#[test]
fn mesh_pipeline_variant_registry_maps_depth_prepass_to_depth_prepass_pass_type() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let variant = registry.resolve_variant(
        MeshPassPipelineKind::DepthPrepass,
        &key,
        ShaderQualityTier::Medium,
    );
    let variant_key = registry.key_for_variant(variant).unwrap();

    assert_eq!(
        variant_key.shader_variant_key().pass_type,
        ShaderPassType::DepthPrepass
    );
}

#[test]
fn mesh_pipeline_variant_registry_maps_hit_proxy_to_dedicated_pass_type() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let variant = registry.resolve_variant(
        MeshPassPipelineKind::HitProxy,
        &key,
        ShaderQualityTier::Medium,
    );
    let variant_key = registry.key_for_variant(variant).unwrap();

    assert_eq!(variant_key.kind(), MeshPassPipelineKind::HitProxy);
    assert_eq!(
        variant_key.shader_variant_key().pass_type,
        ShaderPassType::HitProxy
    );
}

#[test]
fn mesh_pipeline_variant_registry_maps_deferred_gbuffer_to_gbuffer_pass_type() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let variant = registry.resolve_variant_for_geometry(
        MeshPassPipelineKind::GBuffer,
        &key,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ShaderQualityTier::High,
    );
    let variant_key = registry.key_for_variant(variant).unwrap();

    assert_eq!(variant_key.kind(), MeshPassPipelineKind::GBuffer);
    assert_eq!(
        variant_key.shader_variant_key().pass_type,
        ShaderPassType::GBuffer
    );
    assert_eq!(
        variant_key.shader_variant_key().geometry_source,
        GEOMETRY_SOURCE_ID_SKINNED_MESH
    );
    assert_eq!(
        variant_key.shader_variant_key().quality,
        ShaderQualityTier::High
    );
}

#[test]
fn mesh_pipeline_variant_registry_maps_taa_reactive_to_taa_reactive_pass_type() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let reactive_variant = registry.resolve_variant(
        MeshPassPipelineKind::TaaReactiveMask,
        &key,
        ShaderQualityTier::Medium,
    );
    let material_variant = registry.resolve_variant(
        MeshPassPipelineKind::TaaReactiveMaterialMask,
        &key,
        ShaderQualityTier::Medium,
    );
    let reactive_key = registry
        .key_for_variant(reactive_variant)
        .expect("TAA reactive mask variant key");
    let material_key = registry
        .key_for_variant(material_variant)
        .expect("TAA reactive material mask variant key");

    assert_eq!(
        reactive_key.shader_variant_key().pass_type,
        ShaderPassType::TaaReactiveMask
    );
    assert_eq!(
        material_key.shader_variant_key().pass_type,
        ShaderPassType::TaaReactiveMask
    );
    assert_ne!(reactive_variant, material_variant);
}

#[test]
fn mesh_pipeline_variant_registry_counts_variant_misses_and_memory_hits() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    let first =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    let second =
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    assert_eq!(first, second);

    let report = registry.miss_report();
    assert_eq!(report.request_count, 2);
    assert_eq!(report.compile_miss_count, 0);
    assert_eq!(report.memory_hit_count, 1);

    let variant_key = registry
        .key_for_variant(first)
        .expect("base variant key")
        .shader_variant_key()
        .clone();
    registry.record_compile_miss(&variant_key);
    assert_eq!(registry.miss_report().compile_miss_count, 1);

    registry.reset_miss_report();
    let reset_report = registry.miss_report();
    assert_eq!(reset_report.request_count, 0);
    assert_eq!(reset_report.compile_miss_count, 0);
    assert_eq!(reset_report.registered_pipeline_variant_count, 1);
    assert_eq!(reset_report.registered_shader_variant_count, 1);
    assert_eq!(
        reset_report.texture_presence_normalized_pipeline_variant_count,
        1
    );
    assert_eq!(
        reset_report.texture_presence_equivalent_pipeline_variant_count,
        0
    );
}

#[test]
fn mesh_pipeline_variant_registry_attributes_exact_pipeline_targets_without_rescanning() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();
    registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    registry.resolve_variant(
        MeshPassPipelineKind::ShadowDepth,
        &key,
        ShaderQualityTier::Medium,
    );
    let mut alpha_key = key.clone();
    alpha_key.alpha_mask = true;
    registry.resolve_variant(
        MeshPassPipelineKind::ShadowDepthAlphaMask,
        &alpha_key,
        ShaderQualityTier::Medium,
    );
    registry.resolve_variant(
        MeshPassPipelineKind::ShadowDepth,
        &key,
        ShaderQualityTier::Medium,
    );

    for report in [registry.miss_report(), {
        registry.reset_miss_report();
        registry.miss_report()
    }] {
        assert_eq!(
            report
                .pipeline_target_metrics(ShaderPipelineTarget::Base)
                .registered_pipeline_variant_count,
            1
        );
        assert_eq!(
            report
                .pipeline_target_metrics(ShaderPipelineTarget::ShadowDepth)
                .registered_pipeline_variant_count,
            1
        );
        assert_eq!(
            report
                .pipeline_target_metrics(ShaderPipelineTarget::ShadowDepthAlphaMask)
                .registered_pipeline_variant_count,
            1
        );
        assert_eq!(
            report
                .pipeline_target_metrics(ShaderPipelineTarget::Oit)
                .registered_pipeline_variant_count,
            0
        );
    }
}

#[test]
fn repeated_material_binding_requests_reuse_one_pipeline_variant() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();

    for _ in 0_u8..16 {
        registry.resolve_variant(MeshPassPipelineKind::Base, &key, ShaderQualityTier::Medium);
    }

    let report = registry.miss_report();
    assert_eq!(report.request_count, 16);
    assert_eq!(report.memory_hit_count, 15);
    assert_eq!(report.registered_pipeline_variant_count, 1);
    assert_eq!(report.registered_shader_variant_count, 1);
    assert_eq!(report.texture_presence_normalized_pipeline_variant_count, 1);
    assert_eq!(report.texture_presence_equivalent_pipeline_variant_count, 0);
}

#[test]
fn texture_presence_normalization_preserves_distinct_pipeline_kinds_and_shader_features() {
    let mut registry = MeshPipelineVariantRegistry::default();
    let key = default_pipeline_key();
    registry.resolve_variant(
        MeshPassPipelineKind::TaaReactiveMask,
        &key,
        ShaderQualityTier::Medium,
    );
    registry.resolve_variant(
        MeshPassPipelineKind::TaaReactiveMaterialMask,
        &key,
        ShaderQualityTier::Medium,
    );

    let mut normal_mapped = key;
    normal_mapped.has_normal_texture = true;
    registry.resolve_variant(
        MeshPassPipelineKind::Base,
        &normal_mapped,
        ShaderQualityTier::Medium,
    );

    let report = registry.miss_report();
    assert_eq!(report.registered_pipeline_variant_count, 3);
    assert_eq!(report.registered_shader_variant_count, 2);
    assert_eq!(report.texture_presence_normalized_pipeline_variant_count, 3);
    assert_eq!(report.texture_presence_equivalent_pipeline_variant_count, 0);
}
