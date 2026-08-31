mod directory;
mod pathing;
mod platform;
mod recovery;
mod transaction;

use std::fs;
use std::io;
use std::path::Path;

#[cfg(test)]
use directory::sync_new_directory;
pub use directory::{ensure_parent_directories, sync_parent_directory};
pub use pathing::is_atomic_write_transaction_path;
pub use recovery::recover_missing_target_from_backup;
pub(crate) use transaction::publish_staged_file_for_transaction;
pub use transaction::replace_staged_file;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PathEntry {
    Missing,
    File,
    Directory,
    Other,
}

pub(super) fn path_entry(path: &Path) -> io::Result<PathEntry> {
    classify_path_metadata(fs::symlink_metadata(path))
}

fn classify_path_metadata(metadata: io::Result<fs::Metadata>) -> io::Result<PathEntry> {
    match metadata {
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(PathEntry::Other),
        Ok(metadata) if metadata.is_file() => Ok(PathEntry::File),
        Ok(metadata) if metadata.is_dir() => Ok(PathEntry::Directory),
        Ok(_) => Ok(PathEntry::Other),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(PathEntry::Missing),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicWriteFault {
    None,
    Write,
    Sync,
    Replace,
    #[cfg(windows)]
    ReplaceAfterBackup,
    BackupSync,
    CreatedDirectorySync,
    #[cfg(windows)]
    CommittedSync,
    ParentSync,
}

/// Writes bytes to a sibling staging file and atomically replaces the target on commit.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_write_with_fault(path, bytes, AtomicWriteFault::None)
}

/// Writes bytes to a sibling staging file and atomically publishes a new target.
///
/// This operation never replaces an existing target. Callers that allocate user-visible copy
/// names can retry with another candidate when it reports [`io::ErrorKind::AlreadyExists`].
pub fn atomic_write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    transaction::stage_atomic_write(path, bytes)?.commit_new()
}

pub fn atomic_write_with_fault(
    path: &Path,
    bytes: &[u8],
    fault: AtomicWriteFault,
) -> io::Result<()> {
    transaction::stage_atomic_write_with_fault(path, bytes, fault)?.commit()
}

pub type PendingAtomicWrite = transaction::PendingAtomicWrite;

pub fn stage_atomic_write(path: &Path, bytes: &[u8]) -> io::Result<PendingAtomicWrite> {
    transaction::stage_atomic_write(path, bytes)
}

#[cfg(test)]
mod tests;
