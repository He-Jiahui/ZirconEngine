use std::path::Path;

use image::GenericImageView;

use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, TextureAsset};
use crate::asset::{AssetImportError, AssetUri};

impl AssetImporter {
    pub(super) fn import_texture(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let image = image::open(source_path).map_err(|error| {
            AssetImportError::Parse(format!("decode image {}: {error}", source_path.display()))
        })?;
        let rgba = image.to_rgba8();
        let (width, height) = image.dimensions();
        Ok(ImportedAsset::Texture(TextureAsset {
            uri: uri.clone(),
            width,
            height,
            rgba: rgba.into_raw(),
        }))
    }
}
