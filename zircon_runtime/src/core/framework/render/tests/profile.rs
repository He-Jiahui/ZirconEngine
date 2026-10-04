use super::{RenderProductFeature, RenderProfileBundle, RenderProfileValidationError};
use crate::core::framework::render::{
    RenderCapabilityKind, RenderCapabilityMismatchDetail, RenderCapabilitySummary,
    RenderSubmissionConfig,
};

#[test]
fn default_render_requires_screen_space_anti_alias_capability() {
    let capabilities = RenderCapabilitySummary {
        backend_name: "profile-aa-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: false,
        ..RenderCapabilitySummary::default()
    };

    let error = RenderProfileBundle::default_render()
        .validate_capabilities(&capabilities)
        .unwrap_err();

    assert_eq!(
        error,
        RenderProfileValidationError::MissingBackendCapability {
            profile: super::RenderProductProfile::DefaultRender,
            detail: RenderCapabilityMismatchDetail::new(RenderCapabilityKind::ScreenSpaceAntiAlias,),
        }
    );
}

#[test]
fn default_render_accepts_auto_to_fxaa_capable_backend() {
    let capabilities = RenderCapabilitySummary {
        backend_name: "profile-aa-test".to_string(),
        supports_offscreen: true,
        supports_fxaa: true,
        max_supported_msaa_samples: 1,
        ..RenderCapabilitySummary::default()
    };

    let bundle = RenderProfileBundle::default_render();

    assert!(bundle.has_feature(RenderProductFeature::AntiAlias));
    bundle.validate_capabilities(&capabilities).unwrap();
}

#[test]
fn render_profile_bundle_exposes_explicit_pipelined_submission() {
    let bundle = RenderProfileBundle::default_render()
        .with_submission_config(RenderSubmissionConfig::pipelined());

    assert_eq!(
        bundle.submission_config(),
        RenderSubmissionConfig::pipelined()
    );
}

#[test]
fn legacy_profile_bundle_deserialization_defaults_to_synchronous_submission() {
    let mut serialized = serde_json::to_value(RenderProfileBundle::default_render())
        .expect("render profile bundle should serialize");
    serialized
        .as_object_mut()
        .expect("render profile bundle should serialize as an object")
        .remove("submission_config");

    let restored: RenderProfileBundle = serde_json::from_value(serialized)
        .expect("legacy render profile bundle should deserialize");

    assert_eq!(
        restored.submission_config(),
        RenderSubmissionConfig::synchronous()
    );
}

#[test]
fn solari_experimental_requires_bevy_solari_binding_array_caps() {
    let capabilities = RenderCapabilitySummary {
        backend_name: "profile-solari-test".to_string(),
        supports_fxaa: true,
        virtual_geometry_supported: true,
        hybrid_global_illumination_supported: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_buffer_readback: true,
        acceleration_structures_supported: true,
        inline_ray_query: true,
        supports_texture_binding_array: true,
        supports_non_uniform_resource_indexing: true,
        supports_partially_bound_binding_array: true,
        ..RenderCapabilitySummary::default()
    };

    let error = RenderProfileBundle::solari_experimental()
        .validate_capabilities(&capabilities)
        .unwrap_err();

    assert_eq!(
        error,
        RenderProfileValidationError::MissingBackendCapability {
            profile: super::RenderProductProfile::SolariExperimental,
            detail: RenderCapabilityMismatchDetail::new(RenderCapabilityKind::BufferBindingArray,),
        }
    );
}
