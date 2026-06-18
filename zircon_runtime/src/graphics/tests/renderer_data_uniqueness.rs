use crate::graphics::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererDataDocument,
    RendererDataDocumentError, RendererFeatureAsset,
};

#[test]
fn renderer_data_document_rejects_duplicate_stages_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "duplicate-stages"
stages = ["PostProcess", "Ui", "PostProcess"]

[[features]]
name = "PostProcess"
source = "PostProcess"
enabled = true
"#,
    )
    .unwrap();

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderPassStage {
            stage: RenderPassStage::PostProcess
        }
    );
}

#[test]
fn renderer_data_document_rejects_duplicate_features_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "duplicate-features"
stages = ["PostProcess"]

[[features]]
name = "Bloom"
source = "Bloom"
enabled = true

[[features]]
name = "Bloom"
source = "Bloom"
enabled = false
"#,
    )
    .unwrap();

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderFeature {
            feature: "Bloom".to_string()
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_duplicate_stages() {
    let error = runtime_uniqueness_error(
        vec![RenderPassStage::Opaque3d, RenderPassStage::Opaque3d],
        vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)],
    );

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderPassStage {
            stage: RenderPassStage::Opaque3d,
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_duplicate_features() {
    let error = runtime_uniqueness_error(
        vec![RenderPassStage::Opaque3d],
        vec![
            RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh),
            RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh),
        ],
    );

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderFeature {
            feature: "Mesh".to_string(),
        }
    );
}

fn runtime_uniqueness_error(
    stages: Vec<RenderPassStage>,
    features: Vec<RendererFeatureAsset>,
) -> RendererDataDocumentError {
    let renderer = RendererAsset {
        name: "duplicate-runtime-renderer".to_string(),
        stages,
        features,
    };

    RendererDataDocument::from_renderer_asset(&renderer).unwrap_err()
}
