use std::path::Path;

use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2AssetKind, UI_V2_ASSET_SCHEMA_VERSION,
};

#[derive(Default)]
pub struct UiV2AssetLoader;

#[derive(Default)]
pub struct UiZuiAssetLoader;

impl UiV2AssetLoader {
    pub fn load_toml_str(input: &str) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let document: UiV2AssetDocument =
            toml::from_str(input).map_err(|error| UiV2AssetError::ParseToml(error.to_string()))?;
        validate_version(&document)?;
        Ok(document)
    }

    pub fn load_toml_file<P: AsRef<Path>>(path: P) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let input =
            std::fs::read_to_string(path).map_err(|error| UiV2AssetError::Io(error.to_string()))?;
        Self::load_toml_str(&input)
    }
}

impl UiZuiAssetLoader {
    pub fn load_zui_str(input: &str) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let document = UiV2AssetLoader::load_toml_str(input)?;
        validate_zui_component_profile(&document)?;
        Ok(document)
    }

    pub fn load_zui_file<P: AsRef<Path>>(path: P) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let input =
            std::fs::read_to_string(path).map_err(|error| UiV2AssetError::Io(error.to_string()))?;
        Self::load_zui_str(&input)
    }
}

fn validate_version(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    if document.asset.version != UI_V2_ASSET_SCHEMA_VERSION {
        return Err(UiV2AssetError::UnsupportedSchemaVersion {
            asset_id: document.asset.id.clone(),
            version: document.asset.version,
            expected: UI_V2_ASSET_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn validate_zui_component_profile(document: &UiV2AssetDocument) -> Result<(), UiV2AssetError> {
    let asset_id = document.asset.id.clone();
    if document.asset.kind != UiV2AssetKind::Component {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id,
            detail: ".zui assets require asset.kind = \"component\"".to_string(),
        });
    }
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
