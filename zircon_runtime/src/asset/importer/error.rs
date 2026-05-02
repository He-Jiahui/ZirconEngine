use thiserror::Error;

use super::AssetImporterRegistryError;
use crate::core::resource::ResourceLocatorError;

#[derive(Debug, Error)]
pub enum AssetImportError {
    #[error("asset I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("asset uri error: {0}")]
    Uri(#[from] ResourceLocatorError),
    #[error("asset parse failed: {0}")]
    Parse(String),
    #[error("unsupported asset format: {0}")]
    UnsupportedFormat(String),
    #[error("wgsl validation failed: {0}")]
    ShaderValidation(String),
    #[error("asset schema migration failed: {0}")]
    SchemaMigration(String),
    #[error("native asset importer failed: {0}")]
    Native(String),
    #[error("asset importer registry failed: {0}")]
    Registry(String),
    #[error("asset serialization failed: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl From<AssetImporterRegistryError> for AssetImportError {
    fn from(error: AssetImporterRegistryError) -> Self {
        Self::Registry(error.to_string())
    }
}
