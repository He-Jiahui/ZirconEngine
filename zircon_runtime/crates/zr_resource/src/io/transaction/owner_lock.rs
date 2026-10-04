//! 提交、探测与恢复共用同一文件锁；锁文件保留在磁盘上，避免删除后新旧 inode 各自被锁而失去串行性。
//! Cross-process serialization for one durable journal owner.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

use super::error::{DurableTransactionError, TransactionPhase};
use crate::io::sync_parent_directory;

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

pub(super) fn owner_lock_path(directory: &Path) -> io::Result<PathBuf> {
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
#[path = "tests/owner_lock.rs"]
mod tests;
