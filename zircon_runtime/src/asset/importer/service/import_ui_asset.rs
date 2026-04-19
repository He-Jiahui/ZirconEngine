use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset};
use crate::asset::AssetImportError;

impl AssetImporter {
    pub(super) fn import_ui_asset(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        if let Ok(asset) = UiLayoutAsset::from_toml_str(&document) {
            return Ok(ImportedAsset::UiLayout(asset));
        }
        if let Ok(asset) = UiWidgetAsset::from_toml_str(&document) {
            return Ok(ImportedAsset::UiWidget(asset));
        }
        if let Ok(asset) = UiStyleAsset::from_toml_str(&document) {
            return Ok(ImportedAsset::UiStyle(asset));
        }
        Err(AssetImportError::Parse(format!(
            "parse ui asset toml {}: unsupported or mismatched [asset.kind]",
            source_path.display()
        )))
    }
}
