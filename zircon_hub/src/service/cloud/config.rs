use crate::service::error::ServiceError;
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudConfig {
    pub root: PathBuf,
    pub key_file: PathBuf,
}

impl CloudConfig {
    pub fn validate(&self) -> Result<(), ServiceError> {
        if !self.root.is_absolute()
            || !self.key_file.is_absolute()
            || self
                .root
                .components()
                .any(|part| matches!(part, Component::ParentDir))
            || self
                .key_file
                .components()
                .any(|part| matches!(part, Component::ParentDir))
            || self.key_file.starts_with(&self.root)
        {
            return Err(ServiceError::Configuration);
        }
        Ok(())
    }
}

pub(super) fn reject_link(path: &Path) -> Result<(), ServiceError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| ServiceError::Storage)?;
    if metadata.file_type().is_symlink() {
        return Err(ServiceError::Configuration);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // Junctions and other reparse points must not redirect this storage authority.
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(ServiceError::Configuration);
        }
    }
    Ok(())
}
