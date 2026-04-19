use thiserror::Error;

use zircon_resource::ResourceLocatorError;

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
    #[error("asset serialization failed: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
