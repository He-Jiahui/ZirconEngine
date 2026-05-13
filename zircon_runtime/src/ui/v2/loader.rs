use std::path::Path;

use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UI_V2_ASSET_SCHEMA_VERSION,
};

#[derive(Default)]
pub struct UiV2AssetLoader;

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
