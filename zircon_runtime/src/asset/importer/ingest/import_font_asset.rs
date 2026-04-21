use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::{FontAsset, ImportedAsset};
use crate::asset::AssetImportError;

impl AssetImporter {
    pub(super) fn import_font_asset(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        let asset = FontAsset::from_toml_str(&document)
            .map_err(|error| AssetImportError::Parse(format!("parse font toml: {error}")))?;
        Ok(ImportedAsset::Font(asset))
    }
}
