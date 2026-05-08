use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::{
    asset_kind_for_imported_asset, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetUri, ImportedAsset,
};

impl AssetImporter {
    pub fn descriptor_for_source(
        &self,
        source_path: &Path,
    ) -> Result<AssetImporterDescriptor, AssetImportError> {
        self.registry().descriptor_for_source(source_path)
    }

    pub fn import_from_source(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        self.import_with_settings(source_path, uri, toml::Table::new())
            .and_then(|outcome| {
                outcome
                    .root_entry()
                    .map(|entry| entry.asset.clone())
                    .ok_or_else(|| AssetImportError::Parse(format!("missing root asset for {uri}")))
            })
    }

    pub fn import_with_settings(
        &self,
        source_path: &Path,
        uri: &AssetUri,
        import_settings: toml::Table,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        let source_bytes = fs::read(source_path)?;
        self.import_bytes(source_path, uri, source_bytes, import_settings)
    }

    pub fn import_bytes(
        &self,
        source_path: &Path,
        uri: &AssetUri,
        source_bytes: Vec<u8>,
        import_settings: toml::Table,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        let importer = self.registry().select(source_path)?;
        let descriptor = importer.descriptor().clone();
        let context = AssetImportContext::new(
            source_path.to_path_buf(),
            uri.clone(),
            source_bytes,
            import_settings,
        );
        let outcome = importer.import(&context)?;
        if outcome.entries.is_empty() {
            return Err(AssetImportError::Parse(format!(
                "asset importer {} returned no imported asset entries",
                descriptor.id
            )));
        }
        for entry in &outcome.entries {
            let actual_kind = asset_kind_for_imported_asset(&entry.asset);
            if !descriptor.allows_output_kind(actual_kind) {
                return Err(AssetImportError::Parse(format!(
                    "asset importer {} returned {actual_kind:?}, expected {:?}",
                    descriptor.id, descriptor.output_kind
                )));
            }
        }
        Ok(outcome)
    }
}
