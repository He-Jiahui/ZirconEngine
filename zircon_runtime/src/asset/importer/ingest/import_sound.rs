use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, SoundAsset};
use crate::asset::{AssetImportError, AssetUri};

impl AssetImporter {
    pub(super) fn import_sound(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let bytes = fs::read(source_path)?;
        let asset = SoundAsset::from_wav_bytes(uri, &bytes).map_err(|error| {
            AssetImportError::Parse(format!("decode wav {}: {error}", source_path.display()))
        })?;
        Ok(ImportedAsset::Sound(asset))
    }
}
