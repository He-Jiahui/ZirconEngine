use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, MaterialAsset};
use crate::asset::AssetImportError;

impl AssetImporter {
    pub(super) fn import_material(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        let material = MaterialAsset::from_toml_str(&document)
            .map_err(|error| AssetImportError::Parse(format!("parse material toml: {error}")))?;
        Ok(ImportedAsset::Material(material))
    }
}
