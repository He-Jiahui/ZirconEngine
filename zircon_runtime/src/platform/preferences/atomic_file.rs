use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::core::framework::platform::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};
use crate::foundation::persistence::atomic_file::atomic_write;

use super::{PreferenceBackendWorkAuthority, PreferenceStorageBackend};

const BACKEND_NAME: &str = "atomic_file";
const STORAGE_DIRECTORY: &str = "preferences-v1";
const STORAGE_EXTENSION: &str = "zrpref";

#[derive(Clone, Debug)]
pub struct AtomicFilePreferenceStorageBackend {
    root: PathBuf,
}

impl AtomicFilePreferenceStorageBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn storage_path(&self, key: &PreferenceKey) -> PathBuf {
        self.root
            .join(STORAGE_DIRECTORY)
            .join(storage_component(key.namespace()))
            .join(format!(
                "{}.{}",
                storage_component(key.key()),
                STORAGE_EXTENSION
            ))
    }
}

impl PreferenceStorageBackend for AtomicFilePreferenceStorageBackend {
    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        PreferenceStorageBackendKind::AtomicFile
    }

    fn open_read(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<Option<Box<dyn io::Read + Send>>, PreferenceStorageError> {
        match File::open(self.storage_path(key)) {
            Ok(value) => Ok(Some(Box::new(value))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(map_io_error(PreferenceStorageOperation::Read, error)),
        }
    }

    fn write(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
        value: &[u8],
    ) -> Result<(), PreferenceStorageError> {
        let path = self.storage_path(key);
        atomic_write(&path, value)
            .and_then(|()| sync_committed_value(&path))
            .map_err(|error| map_io_error(PreferenceStorageOperation::Write, error))
    }

    fn remove(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError> {
        let path = self.storage_path(key);
        match fs::remove_file(&path) {
            Ok(()) => sync_parent_directory(&path)
                .map_err(|error| map_io_error(PreferenceStorageOperation::Remove, error)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(map_io_error(PreferenceStorageOperation::Remove, error)),
        }
    }

    fn flush(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError> {
        // atomic_write synchronizes each committed value before returning.
        Ok(())
    }
}

fn storage_component(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

fn sync_committed_value(path: &Path) -> io::Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    sync_parent_directory(path)
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    // Windows uses the shared ReplaceFileW commit path after the staged file is synced.
    Ok(())
}

fn map_io_error(operation: PreferenceStorageOperation, error: io::Error) -> PreferenceStorageError {
    let kind = match error.kind() {
        io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => {
            PreferenceStorageErrorKind::Denied
        }
        io::ErrorKind::StorageFull | io::ErrorKind::FileTooLarge | io::ErrorKind::QuotaExceeded => {
            PreferenceStorageErrorKind::CapacityExceeded
        }
        io::ErrorKind::InvalidData
        | io::ErrorKind::NotADirectory
        | io::ErrorKind::IsADirectory
        | io::ErrorKind::AlreadyExists => PreferenceStorageErrorKind::CorruptBackend,
        _ => PreferenceStorageErrorKind::TransientIo,
    };
    PreferenceStorageError::from_source(kind, operation, BACKEND_NAME, error)
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use crate::core::framework::platform::{
        PreferenceStorageErrorKind, PreferenceStorageOperation,
    };

    use super::map_io_error;

    #[test]
    fn platform_preference_storage_maps_host_io_error_categories() {
        let cases = [
            (
                io::ErrorKind::PermissionDenied,
                PreferenceStorageErrorKind::Denied,
            ),
            (
                io::ErrorKind::StorageFull,
                PreferenceStorageErrorKind::CapacityExceeded,
            ),
            (
                io::ErrorKind::FileTooLarge,
                PreferenceStorageErrorKind::CapacityExceeded,
            ),
            (
                io::ErrorKind::QuotaExceeded,
                PreferenceStorageErrorKind::CapacityExceeded,
            ),
            (
                io::ErrorKind::ReadOnlyFilesystem,
                PreferenceStorageErrorKind::Denied,
            ),
            (
                io::ErrorKind::InvalidData,
                PreferenceStorageErrorKind::CorruptBackend,
            ),
            (
                io::ErrorKind::Other,
                PreferenceStorageErrorKind::TransientIo,
            ),
        ];

        for (host_kind, expected) in cases {
            let error = map_io_error(
                PreferenceStorageOperation::Write,
                io::Error::new(host_kind, "injected preference storage failure"),
            );
            assert_eq!(error.kind(), expected);
            assert_eq!(error.operation(), PreferenceStorageOperation::Write);
            assert_eq!(error.backend(), "atomic_file");
            assert!(error.source().is_some());
        }
    }
}
