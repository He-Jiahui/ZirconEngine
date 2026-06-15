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

pub(super) fn rename_pack_file(
    source: &Path,
    destination: &Path,
) -> Result<(), ZrPackDeltaInstallError> {
    create_parent_dir(destination)?;
    fs::rename(source, destination).map_err(|error| ZrPackDeltaInstallError::RenameFailed {
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
        error: error.to_string(),
    })
}

pub(super) fn copy_pack_file(
    source: &Path,
    destination: &Path,
) -> Result<(), ZrPackDeltaInstallError> {
    create_parent_dir(destination)?;
    fs::copy(source, destination).map(|_| ()).map_err(|error| {
        ZrPackDeltaInstallError::WriteFailed {
            path: destination.to_path_buf(),
            error: error.to_string(),
        }
    })
}

pub(super) fn remove_pack_file(path: &Path) -> Result<(), ZrPackDeltaInstallError> {
    fs::remove_file(path).map_err(|error| ZrPackDeltaInstallError::WriteFailed {
        path: path.to_path_buf(),
        error: error.to_string(),
    })
}

pub(super) fn optional_backup_path(path: Option<impl AsRef<Path>>) -> Option<PathBuf> {
    path.map(|path| path.as_ref().to_path_buf())
}
