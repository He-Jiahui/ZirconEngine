use crate::core::framework::render::{
    GBufferChannelMask, ShadingModelDescriptor, SHADING_MODEL_ID_BLINN_PHONG,
    SHADING_MODEL_ID_STANDARD_PBR, SHADING_MODEL_ID_UNLIT,
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

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderMaterialLightingModel, SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR,
        SHADING_MODEL_ID_UNLIT,
    };

    use super::builtin_shading_model_registry;

    #[test]
    fn builtin_shading_model_registry_contains_three_surface_models() {
        let registry = builtin_shading_model_registry();
        assert_eq!(registry.get(SHADING_MODEL_ID_UNLIT).unwrap().token, "unlit");
        assert_eq!(
            registry.get(SHADING_MODEL_ID_BLINN_PHONG).unwrap().token,
            "blinn_phong"
        );
        assert_eq!(
            registry.get(SHADING_MODEL_ID_STANDARD_PBR).unwrap().token,
            "pbr"
        );
    }

    #[test]
    fn builtin_shading_model_registry_resolves_lighting_model_tokens() {
        let registry = builtin_shading_model_registry();
        assert_eq!(
            registry
                .resolve_lighting_model(&RenderMaterialLightingModel::Pbr)
                .unwrap()
                .id,
            SHADING_MODEL_ID_STANDARD_PBR
        );
        assert_eq!(
            registry
                .resolve_lighting_model(&RenderMaterialLightingModel::BlinnPhong)
                .unwrap()
                .id,
            SHADING_MODEL_ID_BLINN_PHONG
        );
        assert_eq!(
            registry
                .resolve_lighting_model(&RenderMaterialLightingModel::Unlit)
                .unwrap()
                .id,
            SHADING_MODEL_ID_UNLIT
        );
        assert!(registry
            .resolve_lighting_model(&RenderMaterialLightingModel::Custom {
                name: "subsurface".to_string()
            })
            .is_none());
    }
}
