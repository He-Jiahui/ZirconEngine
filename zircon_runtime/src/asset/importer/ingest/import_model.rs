use std::fs;
use std::path::Path;

use super::primitive_from_indexed_mesh::backfill_virtual_geometry_for_model;
use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, ModelAsset};
use crate::asset::AssetImportError;

impl AssetImporter {
    pub(super) fn import_model(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let document = fs::read_to_string(source_path)?;
        let mut model = ModelAsset::from_toml_str(&document)
            .map_err(|error| AssetImportError::Parse(format!("parse model toml: {error}")))?;
        backfill_virtual_geometry_for_model(&mut model);
        Ok(ImportedAsset::Model(model))
    }
}
