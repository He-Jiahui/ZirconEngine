use crate::core::framework::render::{
    GBufferChannelMask, ShadingModelDescriptor, ShadingModelRegistrationError,
    SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR, SHADING_MODEL_ID_UNLIT,
};

use super::registry::ShadingModelRegistry;

pub(crate) fn builtin_shading_model_registry() -> ShadingModelRegistry {
    let mut registry = ShadingModelRegistry::new(GBufferChannelMask::standard_deferred_v1());
    registry
        .register_builtin(ShadingModelDescriptor::new(
            SHADING_MODEL_ID_UNLIT,
            "unlit",
            "zr_shading_unlit",
            "zr_gbuffer_encode_unlit",
            "zr_shade_deferred_unlit",
            GBufferChannelMask::unlit(),
        ))
        .expect("builtin unlit shading model must register");
    registry
        .register_builtin(ShadingModelDescriptor::new(
            SHADING_MODEL_ID_BLINN_PHONG,
            "blinn_phong",
            "zr_shading_blinn_phong",
            "zr_gbuffer_encode_blinn_phong",
            "zr_shade_deferred_blinn_phong",
            GBufferChannelMask::standard_lit(),
        ))
        .expect("builtin Blinn-Phong shading model must register");
    registry
        .register_builtin(ShadingModelDescriptor::new(
            SHADING_MODEL_ID_STANDARD_PBR,
            "pbr",
            "zr_shading_standard_pbr",
            "zr_gbuffer_encode_standard_pbr",
            "zr_shade_deferred_standard_pbr",
            GBufferChannelMask::standard_lit(),
        ))
        .expect("builtin StandardPBR shading model must register");
    registry
}

pub(crate) fn shading_model_registry_with_plugin_descriptors(
    plugin_descriptors: impl IntoIterator<Item = ShadingModelDescriptor>,
) -> Result<ShadingModelRegistry, ShadingModelRegistrationError> {
    let mut registry = builtin_shading_model_registry();
    for descriptor in plugin_descriptors {
        registry.register_plugin_descriptor(descriptor)?;
    }
    Ok(registry)
}

#[cfg(test)]
#[path = "tests/builtins.rs"]
mod tests;
