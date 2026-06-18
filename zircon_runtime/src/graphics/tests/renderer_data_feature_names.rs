use crate::graphics::{RendererDataDocument, RendererDataDocumentError};

#[test]
fn renderer_data_document_rejects_feature_name_aliases_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "feature-name-alias"
stages = ["PostProcess"]

[[features]]
name = "Readable Mesh Label"
source = "Mesh"
enabled = true
"#,
    )
    .unwrap();

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::MismatchedRenderFeatureName {
            name: "Readable Mesh Label".to_string(),
            source: "Mesh".to_string()
        }
    );
}

#[test]
fn renderer_data_document_accepts_canonical_feature_name_source_pair() {
    let renderer = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "canonical-feature-name"
stages = ["PostProcess"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
"#,
    )
    .unwrap()
    .to_renderer_asset()
    .unwrap();

    assert_eq!(renderer.features[0].feature_name(), "Mesh");
}
