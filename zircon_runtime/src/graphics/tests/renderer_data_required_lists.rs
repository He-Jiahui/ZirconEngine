use crate::graphics::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererDataDocument,
    RendererDataDocumentError, RendererFeatureAsset,
};

#[test]
fn renderer_data_document_rejects_empty_stage_lists_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "missing-stages"
stages = []

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
"#,
    )
    .unwrap();

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(error, RendererDataDocumentError::EmptyRenderPassStageList);
}

#[test]
fn renderer_data_document_rejects_empty_feature_lists_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "missing-features"
stages = ["Opaque3d"]
features = []
"#,
    )
    .unwrap();

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(error, RendererDataDocumentError::EmptyRenderFeatureList);
}

#[test]
fn renderer_asset_projection_rejects_empty_stage_lists() {
    let renderer = RendererAsset {
        name: "missing-runtime-stages".to_string(),
        stages: Vec::new(),
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)],
    };

    let error = RendererDataDocument::from_renderer_asset(&renderer).unwrap_err();

    assert_eq!(error, RendererDataDocumentError::EmptyRenderPassStageList);
}

#[test]
fn renderer_asset_projection_rejects_empty_feature_lists() {
    let renderer = RendererAsset {
        name: "missing-runtime-features".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: Vec::new(),
    };

    let error = RendererDataDocument::from_renderer_asset(&renderer).unwrap_err();

    assert_eq!(error, RendererDataDocumentError::EmptyRenderFeatureList);
}
