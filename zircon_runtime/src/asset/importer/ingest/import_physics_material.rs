use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::ImportedAsset;
use crate::asset::{AssetImportError, PhysicsMaterialAsset};

impl AssetImporter {
    pub fn import_physics_material(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        PhysicsMaterialAsset::from_toml_str(&document)
            .map(ImportedAsset::PhysicsMaterial)
            .map_err(|error| AssetImportError::Parse(error.to_string()))
    }
}
