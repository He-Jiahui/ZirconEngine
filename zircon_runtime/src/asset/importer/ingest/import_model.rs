use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, ModelAsset};
use crate::asset::AssetImportError;

impl AssetImporter {
    pub(super) fn import_model(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        let model = ModelAsset::from_toml_str(&document)
            .map_err(|error| AssetImportError::Parse(format!("parse model toml: {error}")))?;
        Ok(ImportedAsset::Model(model))
    }
}
