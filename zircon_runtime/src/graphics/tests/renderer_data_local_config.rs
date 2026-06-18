use crate::graphics::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererDataDocument,
    RendererDataDocumentError, RendererFeatureAsset,
};

#[test]
fn renderer_data_document_rejects_empty_local_config_keys_before_runtime_projection() {
    let error = document_local_config_error(r#""""#);

    assert_eq!(
        error,
        RendererDataDocumentError::EmptyRenderFeatureLocalConfigKey {
            feature: "Mesh".to_string(),
        }
    );
}

#[test]
fn renderer_data_document_rejects_padded_local_config_keys_before_runtime_projection() {
    let error = document_local_config_error(r#"" variant ""#);

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureLocalConfigKey {
            feature: "Mesh".to_string(),
            key: " variant ".to_string(),
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_empty_local_config_keys() {
    let error = runtime_local_config_error("");

    assert_eq!(
        error,
        RendererDataDocumentError::EmptyRenderFeatureLocalConfigKey {
            feature: "Mesh".to_string(),
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_padded_local_config_keys() {
    let error = runtime_local_config_error(" variant ");

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureLocalConfigKey {
            feature: "Mesh".to_string(),
            key: " variant ".to_string(),
        }
    );
}

fn document_local_config_error(key: &str) -> RendererDataDocumentError {
    let document = RendererDataDocument::from_toml_str(&format!(
        r#"
version = 1
name = "local-config-keys"
stages = ["Opaque3d"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
local_config = {{ {key} = "lit" }}
"#
    ))
    .unwrap();

    document.to_renderer_asset().unwrap_err()
}

fn runtime_local_config_error(key: &str) -> RendererDataDocumentError {
    let renderer = RendererAsset {
        name: "local-config-keys".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![
            RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh).with_local_config(key, "lit")
        ],
    };

    RendererDataDocument::from_renderer_asset(&renderer).unwrap_err()
}
