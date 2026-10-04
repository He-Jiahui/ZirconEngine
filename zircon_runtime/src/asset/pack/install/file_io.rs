use std::{
    fs,
    path::{Path, PathBuf},
};

use super::ZrPackDeltaInstallError;

pub(super) fn read_pack_file(path: &Path) -> Result<Vec<u8>, ZrPackDeltaInstallError> {
    fs::read(path).map_err(|error| ZrPackDeltaInstallError::ReadFailed {
        path: path.to_path_buf(),
        error: error.to_string(),
    })
}

pub(super) fn create_parent_dir(path: &Path) -> Result<(), ZrPackDeltaInstallError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| ZrPackDeltaInstallError::WriteFailed {
                path: parent.to_path_buf(),
                error: error.to_string(),
            })?;
        }
    }
    Ok(())
}

pub(super) fn write_pack_file(path: &Path, bytes: &[u8]) -> Result<(), ZrPackDeltaInstallError> {
    create_parent_dir(path)?;
    fs::write(path, bytes).map_err(|error| ZrPackDeltaInstallError::WriteFailed {
        path: path.to_path_buf(),
        error: error.to_string(),
    })
}

pub(super) fn optional_backup_path(path: Option<impl AsRef<Path>>) -> Option<PathBuf> {
    path.map(|path| path.as_ref().to_path_buf())
}
