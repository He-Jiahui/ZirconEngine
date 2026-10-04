use crate::graphics::{
    RendererDataDocument, RendererDataDocumentError, RENDERER_DATA_DOCUMENT_VERSION,
};

#[test]
fn renderer_data_document_rejects_future_versions_before_runtime_projection() {
    let document = RendererDataDocument::from_toml_str(
        r#"
version = 2
name = "future-renderer-data"
stages = ["PostProcess"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
"#,
    )
    .unwrap();

    assert_eq!(document.version, RENDERER_DATA_DOCUMENT_VERSION + 1);

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::UnsupportedDocumentVersion {
            version: RENDERER_DATA_DOCUMENT_VERSION + 1,
            supported: RENDERER_DATA_DOCUMENT_VERSION,
        }
    );
}

#[test]
fn renderer_data_document_uses_current_version_when_field_is_omitted() {
    let document = RendererDataDocument::from_toml_str(
        r#"
name = "implicit-current-version"
stages = ["PostProcess"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
"#,
    )
    .unwrap();

    assert_eq!(document.version, RENDERER_DATA_DOCUMENT_VERSION);
    assert_eq!(document.to_renderer_asset().unwrap().name, document.name);
}
