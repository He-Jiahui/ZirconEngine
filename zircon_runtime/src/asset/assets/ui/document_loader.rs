use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2AssetKind, UI_V2_ASSET_SCHEMA_VERSION,
};

pub(super) fn load_current_ui_document(input: &str) -> Result<UiAssetDocument, UiAssetError> {
    let document: UiAssetDocument =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    if document.asset.version != UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION {
        return Err(unsupported_current_schema_error(
            document.asset.id,
            document.asset.version,
        ));
    }
    Ok(document)
}

pub(super) fn load_ui_v2_document(input: &str) -> Result<UiV2AssetDocument, UiV2AssetError> {
    let document: UiV2AssetDocument =
        toml::from_str(input).map_err(|error| UiV2AssetError::ParseToml(error.to_string()))?;
    validate_version(document)
}

pub(super) fn load_zui_document(input: &str) -> Result<UiV2AssetDocument, UiV2AssetError> {
    let document = load_ui_v2_document(input)?;
    validate_zui_document_profile(&document)?;
    Ok(document)
}

fn validate_version(document: UiV2AssetDocument) -> Result<UiV2AssetDocument, UiV2AssetError> {
    if document.asset.version != UI_V2_ASSET_SCHEMA_VERSION {
        return Err(unsupported_v2_schema_error(
            document.asset.id,
            document.asset.version,
        ));
    }
    Ok(document)
}

fn unsupported_current_schema_error(asset_id: String, version: u32) -> UiAssetError {
    UiAssetError::UnsupportedSchemaVersion {
        asset_id,
        version,
        current: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
    }
}

fn unsupported_v2_schema_error(asset_id: String, version: u32) -> UiV2AssetError {
    UiV2AssetError::UnsupportedSchemaVersion {
        asset_id,
        version,
        expected: UI_V2_ASSET_SCHEMA_VERSION,
    }
}

fn validate_zui_document_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    match document.asset.kind {
        UiV2AssetKind::Component => validate_zui_component_profile(document),
        UiV2AssetKind::View => validate_zui_view_profile(document),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => validate_zui_style_profile(document),
    }
}

fn validate_zui_component_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    let asset_id = document.asset.id.clone();
    if document.root.is_some() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: ".zui component assets must not declare a [root] view entry".to_string(),
        });
    }
    if document.components.len() != 1 {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: format!(
                ".zui component assets must declare exactly one component; found {}",
                document.components.len()
            ),
        });
    }

    let (component_id, component) = document
        .components
        .iter()
        .next()
        .expect("zui component count validated above");
    if component.root.trim().is_empty() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: format!(".zui component {component_id} must declare a non-empty root node"),
        });
    }
    if !document.nodes.contains_key(&component.root) {
        return Err(UiV2AssetError::MissingNode {
            asset_id,
            node_id: component.root.clone(),
        });
    }
    Ok(())
}

fn validate_zui_view_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    let asset_id = document.asset.id.clone();
    let root = document
        .root
        .as_ref()
        .ok_or_else(|| UiV2AssetError::InvalidDocument {
            asset_id: asset_id.clone(),
            detail: ".zui view assets must declare a [root] view entry".to_string(),
        })?;
    if root.node.trim().is_empty() {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: ".zui view assets must declare a non-empty root node".to_string(),
        });
    }
    if !document.nodes.contains_key(&root.node) {
        return Err(UiV2AssetError::MissingNode {
            asset_id,
            node_id: root.node.clone(),
        });
    }
    Ok(())
}

fn validate_zui_style_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    if let Some(root) = &document.root {
        let asset_id = document.asset.id.clone();
        if root.node.trim().is_empty() {
            return Err(UiV2AssetError::InvalidDocument {
                asset_id,
                detail:
                    ".zui style assets must declare a non-empty root node when [root] is present"
                        .to_string(),
            });
        }
        if !document.nodes.contains_key(&root.node) {
            return Err(UiV2AssetError::MissingNode {
                asset_id,
                node_id: root.node.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "document_loader/owned_schema_error_tests.rs"]
mod owned_schema_error_tests;
