use crate::core::framework::render::{
    GBufferChannelMask, RenderMaterialLightingModel, ShadingModelDescriptor, ShadingModelId,
    ShadingModelRegistrationError, SHADING_MODEL_ID_STANDARD_PBR, SHADING_MODEL_PLUGIN_ID_START,
};

use super::ShadingModelRegistry;

#[test]
fn common_shading_model_lookups_use_borrowed_normalized_tokens() {
    let source = include_str!("../registry.rs");

    assert!(source.contains("self.tokens.get(trimmed)"));
    assert!(source.contains("RenderMaterialLightingModel::Pbr => self.resolve_token(\"pbr\")"));
    assert!(!source.contains(concat!("model.", "as_token()")));
}

fn descriptor(id: u8, token: &str) -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        crate::core::framework::render::ShadingModelId::new(id),
        token,
        "forward",
        "gbuffer",
        "deferred",
        GBufferChannelMask::standard_lit(),
    )
}

#[test]
fn shading_model_registry_rejects_duplicate_id() {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
    registry
        .register_builtin(descriptor(SHADING_MODEL_ID_STANDARD_PBR.value(), "pbr"))
        .unwrap();

    let error = registry
        .register_builtin(descriptor(
            SHADING_MODEL_ID_STANDARD_PBR.value(),
            "standard_pbr",
        ))
        .unwrap_err();
    assert!(matches!(
        error,
        ShadingModelRegistrationError::DuplicateId { .. }
    ));
}

#[test]
fn shading_model_registry_rejects_duplicate_token() {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
    registry.register_builtin(descriptor(2, "pbr")).unwrap();

    let error = registry.register_builtin(descriptor(3, "PBR")).unwrap_err();
    assert!(matches!(
        error,
        ShadingModelRegistrationError::DuplicateToken { .. }
    ));
}

#[test]
fn shading_model_registry_resolves_registered_lighting_model_tokens() {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
    registry
        .register_plugin_descriptor(descriptor(
            SHADING_MODEL_PLUGIN_ID_START,
            "custom:subsurface",
        ))
        .unwrap();

    let resolved = registry
        .resolve_lighting_model(&RenderMaterialLightingModel::Custom {
            name: "subsurface".to_string(),
        })
        .unwrap();
    assert_eq!(
        resolved.id,
        ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START)
    );
}

#[test]
fn shading_model_registry_rejects_plugin_descriptor_in_builtin_id_range() {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());

    let error = registry
        .register_plugin_descriptor(descriptor(
            SHADING_MODEL_ID_STANDARD_PBR.value(),
            "custom:subsurface",
        ))
        .unwrap_err();

    assert!(matches!(
        error,
        ShadingModelRegistrationError::PluginIdReserved { .. }
    ));
}

#[test]
fn shading_model_registry_rejects_unsupported_required_channels() {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
    let descriptor = ShadingModelDescriptor::new(
        crate::core::framework::render::ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START),
        "subsurface",
        "forward",
        "gbuffer",
        "deferred",
        GBufferChannelMask::standard_lit().union(GBufferChannelMask::CUSTOM0),
    );
    let error = registry.register_plugin_descriptor(descriptor).unwrap_err();
    assert!(matches!(
        error,
        ShadingModelRegistrationError::RequiredChannelsUnsupported { .. }
    ));
}
