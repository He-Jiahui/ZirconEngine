use thiserror::Error;

use crate::core::resource::ResourceLocator;

pub trait ResourceIo: Send + Sync {
    fn read(&self, locator: &ResourceLocator) -> Result<Vec<u8>, ResourceIoError>;

    fn write(&self, locator: &ResourceLocator, bytes: &[u8]) -> Result<(), ResourceIoError>;

    fn exists(&self, locator: &ResourceLocator) -> bool;
}

#[derive(Debug, Error)]
pub enum ResourceIoError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("resource io failed: {0}")]
    Io(String),
    #[error("resource scheme is read only: {0}")]
    ReadOnly(String),
}
