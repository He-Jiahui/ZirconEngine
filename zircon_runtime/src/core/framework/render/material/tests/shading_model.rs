use super::*;

#[test]
fn render_material_lighting_model_token_resolves_shading_id() {
    assert_eq!(
        ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::Pbr),
        Some(SHADING_MODEL_ID_STANDARD_PBR)
    );
    assert_eq!(
        ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::BlinnPhong),
        Some(SHADING_MODEL_ID_BLINN_PHONG)
    );
    assert_eq!(
        ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::Unlit),
        Some(SHADING_MODEL_ID_UNLIT)
    );
    assert_eq!(
        ShadingModelId::from_lighting_model(&RenderMaterialLightingModel::Custom {
            name: "subsurface".to_string()
        }),
        None
    );
}

#[test]
fn render_material_shading_model_id_roundtrips_gbuffer_encoding() {
    for id in [
        SHADING_MODEL_ID_UNLIT,
        SHADING_MODEL_ID_BLINN_PHONG,
        SHADING_MODEL_ID_STANDARD_PBR,
        ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START),
        ShadingModelId::new(u8::MAX),
    ] {
        assert_eq!(
            ShadingModelId::decode_gbuffer_alpha(id.encode_gbuffer_alpha()),
            id
        );
    }
}

#[test]
fn render_material_custom_lighting_model_waits_for_plugin_registration() {
    assert!(ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START).is_plugin_range());
    assert!(!SHADING_MODEL_ID_STANDARD_PBR.is_plugin_range());
}

#[test]
fn gbuffer_channel_mask_reports_required_channel_overflow() {
    let supported = GBufferChannelMask::standard_deferred_v1();
    assert!(supported.contains(GBufferChannelMask::standard_lit()));
    assert!(!supported.contains(GBufferChannelMask::CUSTOM0));
}
