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
