use crate::graphics::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererDataDocument,
    RendererDataDocumentError, RendererFeatureAsset,
};

#[test]
fn renderer_data_document_rejects_empty_renderer_names_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = ""
stages = ["Opaque3d"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
"#,
    )
    .unwrap();

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(error, RendererDataDocumentError::EmptyRendererDataName);
}

#[test]
fn renderer_data_document_rejects_padded_renderer_names_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = " default-forward "
stages = ["Opaque3d"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
"#,
    )
    .unwrap();

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRendererDataName {
            name: " default-forward ".to_string(),
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_empty_renderer_names() {
    let error = runtime_name_error("");

    assert_eq!(error, RendererDataDocumentError::EmptyRendererDataName);
}

#[test]
fn renderer_asset_projection_rejects_padded_renderer_names() {
    let error = runtime_name_error(" default-forward ");

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRendererDataName {
            name: " default-forward ".to_string(),
        }
    );
}

fn runtime_name_error(name: &str) -> RendererDataDocumentError {
    let renderer = RendererAsset {
        name: name.to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)],
    };

    RendererDataDocument::from_renderer_asset(&renderer).unwrap_err()
}
