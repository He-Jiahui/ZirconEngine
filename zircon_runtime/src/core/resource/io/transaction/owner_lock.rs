//! Cross-process serialization for one durable journal owner.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

use super::error::{DurableTransactionError, TransactionPhase};
use crate::core::resource::io::sync_parent_directory;

#[derive(Debug)]
pub(super) struct TransactionOwnerLock {
    file: File,
}

impl TransactionOwnerLock {
    pub(super) fn acquire(
        directory: &Path,
        phase: TransactionPhase,
    ) -> Result<Self, DurableTransactionError> {
        let path =
            owner_lock_path(directory).map_err(|source| operation(phase, directory, source))?;
        let (file, created) =
            open_lock_file(&path).map_err(|source| operation(phase, &path, source))?;
        if created {
            file.sync_all()
                .and_then(|()| sync_parent_directory(&path))
                .map_err(|source| operation(phase, &path, source))?;
        }
        File::try_lock(&file).map_err(|source| {
            let source = match source {
                TryLockError::WouldBlock => io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "another process owns the durable transaction journal",
                ),
                TryLockError::Error(source) => source,
            };
            operation(phase, &path, source)
        })?;
        Ok(Self { file })
    }
}

impl Drop for TransactionOwnerLock {
    fn drop(&mut self) {
        let _ = File::unlock(&self.file);
    }
}

fn owner_lock_path(directory: &Path) -> io::Result<PathBuf> {
    let parent = directory.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "durable transaction journal has no lock owner",
        )
    })?;
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "durable transaction journal name is not UTF-8",
            )
        })?;
    Ok(parent.join(format!(".{name}.zrlock")))
}

fn open_lock_file(path: &Path) -> io::Result<(File, bool)> {
    match OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => Ok((file, true)),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "durable transaction owner lock must be a regular non-link file",
                ));
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map(|file| (file, false))
        }
        Err(error) => Err(error),
    }
}

fn operation(phase: TransactionPhase, path: &Path, source: io::Error) -> DurableTransactionError {
    DurableTransactionError::operation(phase, path, source)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn owner_lock_rejects_a_second_live_holder() {
        let root = std::env::temp_dir().join(format!(
            "zircon-durable-owner-lock-{}-{}",
            std::process::id(),
            crate::core::resource::io::NEXT_ATOMIC_FILE_ID
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let journal = root.join("journal");
        fs::create_dir_all(&journal).unwrap();
        let first = TransactionOwnerLock::acquire(&journal, TransactionPhase::Stage).unwrap();

        let error = TransactionOwnerLock::acquire(&journal, TransactionPhase::Stage).unwrap_err();

        assert!(error.to_string().contains("another process owns"));
        drop(first);
        TransactionOwnerLock::acquire(&journal, TransactionPhase::Stage).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
