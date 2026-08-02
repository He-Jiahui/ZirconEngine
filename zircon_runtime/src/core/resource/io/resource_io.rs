use crate::core::resource::ResourceLocator;

use super::ResourceIoError;

pub trait ResourceIo: Send + Sync {
    fn read(&self, locator: &ResourceLocator) -> Result<Vec<u8>, ResourceIoError>;

    fn write(&self, locator: &ResourceLocator, bytes: &[u8]) -> Result<(), ResourceIoError>;

    fn exists(&self, locator: &ResourceLocator) -> bool;
}
