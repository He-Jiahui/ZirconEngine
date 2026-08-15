mod directory;
mod pathing;
mod platform;
mod recovery;
mod transaction;

use std::io;
use std::path::Path;
use std::sync::atomic::AtomicU64;

#[cfg(test)]
use directory::sync_new_directory;
pub(crate) use directory::{ensure_parent_directories, sync_parent_directory};
pub(crate) use pathing::is_atomic_write_transaction_path;
pub(crate) use recovery::recover_missing_target_from_backup;
pub(crate) use transaction::replace_staged_file;

pub(crate) static NEXT_ATOMIC_FILE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AtomicWriteFault {
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

pub(crate) fn atomic_write_with_fault(
    path: &Path,
    bytes: &[u8],
    fault: AtomicWriteFault,
) -> io::Result<()> {
    transaction::stage_atomic_write_with_fault(path, bytes, fault)?.commit()
}

pub(crate) type PendingAtomicWrite = transaction::PendingAtomicWrite;

pub(crate) fn stage_atomic_write(path: &Path, bytes: &[u8]) -> io::Result<PendingAtomicWrite> {
    transaction::stage_atomic_write(path, bytes)
}

#[cfg(test)]
mod tests;
