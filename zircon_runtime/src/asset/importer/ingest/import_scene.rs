use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, SceneAsset};
use crate::asset::AssetImportError;

impl AssetImporter {
    pub(super) fn import_scene(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        let scene = SceneAsset::from_toml_str(&document)
            .map_err(|error| AssetImportError::Parse(format!("parse scene toml: {error}")))?;
        Ok(ImportedAsset::Scene(scene))
    }
}
