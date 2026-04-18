use std::fs;
use std::path::Path;

use super::validate_wgsl::validate_wgsl;
use super::AssetImporter;
use crate::assets::{ImportedAsset, ShaderAsset};
use crate::{AssetImportError, AssetUri};

impl AssetImporter {
    pub(super) fn import_shader(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let source = fs::read_to_string(source_path)?;
        validate_wgsl(uri, &source)?;
        Ok(ImportedAsset::Shader(ShaderAsset {
            uri: uri.clone(),
            source,
        }))
    }
}
