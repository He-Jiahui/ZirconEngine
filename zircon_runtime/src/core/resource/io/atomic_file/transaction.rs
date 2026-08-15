use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::directory::{
    create_and_sync_parent_directories, sync_parent_directory, sync_parent_directory_with_fault,
};
use super::pathing::unique_sibling_path;
use super::platform;
use super::AtomicWriteFault;

pub(super) fn stage_atomic_write(path: &Path, bytes: &[u8]) -> io::Result<PendingAtomicWrite> {
    stage_atomic_write_with_fault(path, bytes, AtomicWriteFault::None)
}

pub(super) fn stage_atomic_write_with_fault(
    path: &Path,
    bytes: &[u8],
    fault: AtomicWriteFault,
) -> io::Result<PendingAtomicWrite> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    if let Some(parent) = parent {
        create_and_sync_parent_directories(parent, fault)?;
    }
    let directory = parent.unwrap_or_else(|| Path::new("."));
    let (staging_path, mut staging_file) = create_staging_file(directory, path)?;
    if let Err(error) = write_and_sync(&mut staging_file, bytes, fault) {
        drop(staging_file);
        let _ = fs::remove_file(&staging_path);
        return Err(error);
    }
    drop(staging_file);

    Ok(PendingAtomicWrite {
        target: path.to_path_buf(),
        staging_path,
        fault,
    })
}

#[derive(Debug)]
pub(crate) struct PendingAtomicWrite {
    target: PathBuf,
    staging_path: PathBuf,
    fault: AtomicWriteFault,
}

impl PendingAtomicWrite {
    pub(crate) fn commit(self) -> io::Result<()> {
        let path = self.target.as_path();
        let staging_path = self.staging_path.as_path();

        if !path.exists() {
            if should_fail_before_commit(self.fault) {
                let _ = fs::remove_file(staging_path);
                return Err(injected_commit_error());
            }
            return rename_staging(staging_path, path, self.fault);
        }

        let directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let backup_path = unique_sibling_path(directory, path, "backup");
        commit_replace(path, staging_path, &backup_path, self.fault)
    }
}

impl Drop for PendingAtomicWrite {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.staging_path);
    }
}

fn write_and_sync(file: &mut File, bytes: &[u8], fault: AtomicWriteFault) -> io::Result<()> {
    if fault == AtomicWriteFault::Write {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic file write failure",
        ));
    }
    file.write_all(bytes)?;
    file.flush()?;
    if fault == AtomicWriteFault::Sync {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic file sync failure",
        ));
    }
    file.sync_all()
}

fn create_staging_file(directory: &Path, target: &Path) -> io::Result<(PathBuf, File)> {
    loop {
        let path = unique_sibling_path(directory, target, "staging");
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

fn rename_staging(staging_path: &Path, target: &Path, fault: AtomicWriteFault) -> io::Result<()> {
    match platform::rename_staging(staging_path, target) {
        Ok(()) => sync_parent_directory_with_fault(target, fault),
        Err(error) => {
            let _ = fs::remove_file(staging_path);
            Err(error)
        }
    }
}

#[cfg(unix)]
fn commit_replace(
    target: &Path,
    staging_path: &Path,
    backup_path: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    if let Err(link_error) = fs::hard_link(target, backup_path) {
        if let Err(copy_error) = fs::copy(target, backup_path) {
            let _ = fs::remove_file(staging_path);
            return Err(io::Error::new(
                copy_error.kind(),
                format!(
                    "failed to preserve atomic file backup: hard-link failed: {link_error}; copy failed: {copy_error}"
                ),
            ));
        }
    }
    let backup_sync = if should_fail_backup_sync(fault) {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic file backup sync failure",
        ))
    } else {
        File::open(backup_path).and_then(|file| file.sync_all())
    };
    if let Err(error) = backup_sync {
        let _ = fs::remove_file(staging_path);
        return Err(cleanup_precommit_backup_after_error(backup_path, error));
    }
    if let Err(error) = sync_parent_directory(backup_path) {
        let _ = fs::remove_file(staging_path);
        return Err(cleanup_precommit_backup_after_error(backup_path, error));
    }
    if should_fail_before_commit(fault) {
        let _ = fs::remove_file(staging_path);
        return Err(cleanup_precommit_backup_after_error(
            backup_path,
            injected_commit_error(),
        ));
    }
    if let Err(error) = platform::rename_staging(staging_path, target) {
        let _ = fs::remove_file(staging_path);
        return Err(cleanup_precommit_backup_after_error(backup_path, error));
    }
    sync_parent_directory_with_fault(target, fault)?;
    remove_committed_backup(backup_path)
}

#[cfg(windows)]
fn commit_replace(
    target: &Path,
    staging_path: &Path,
    backup_path: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    if should_fail_before_commit(fault) {
        let _ = fs::remove_file(staging_path);
        return Err(injected_commit_error());
    }
    if fault == AtomicWriteFault::ReplaceAfterBackup {
        platform::rename_staging(target, backup_path)?;
        return super::recovery::handle_windows_replace_failure(
            target,
            staging_path,
            backup_path,
            injected_commit_error(),
        );
    }
    if let Err(error) = platform::replace_file_with_backup(target, staging_path, backup_path) {
        return super::recovery::handle_windows_replace_failure(
            target,
            staging_path,
            backup_path,
            error,
        );
    }
    sync_windows_committed_file(target, fault)?;
    sync_parent_directory_with_fault(target, fault)?;
    // The replacement and backup have committed atomically. Cleanup is post-commit.
    remove_committed_backup(backup_path)
}

#[cfg(windows)]
fn sync_windows_committed_file(path: &Path, fault: AtomicWriteFault) -> io::Result<()> {
    if fault == AtomicWriteFault::CommittedSync {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic file committed-target sync failure",
        ));
    }
    platform::sync_committed_target(path)
}

#[cfg(not(any(unix, windows)))]
fn commit_replace(
    _target: &Path,
    staging_path: &Path,
    _backup_path: &Path,
    _fault: AtomicWriteFault,
) -> io::Result<()> {
    let _ = fs::remove_file(staging_path);
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "atomic file replacement is unsupported on this platform",
    ))
}

fn cleanup_precommit_backup_after_error(backup_path: &Path, error: io::Error) -> io::Error {
    match remove_file_and_sync_parent(backup_path) {
        Ok(()) => error,
        Err(cleanup_error) => io::Error::new(
            error.kind(),
            format!(
                "{error}; failed to durably remove pre-commit backup {}: {cleanup_error}",
                backup_path.display()
            ),
        ),
    }
}

fn remove_committed_backup(backup_path: &Path) -> io::Result<()> {
    remove_file_and_sync_parent(backup_path)
}

fn remove_file_and_sync_parent(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => sync_parent_directory(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Replaces a target with an already-synchronized sibling file.
///
/// Durable multi-file transactions retain their own backups until the journal reaches a terminal
/// phase, so this primitive deliberately does not create or consume an additional backup.
pub(crate) fn replace_staged_file(staging: &Path, target: &Path) -> io::Result<()> {
    if target.exists() {
        platform::replace_existing_staged_file(staging, target)?;
    } else {
        platform::rename_staging(staging, target)?;
    }
    platform::sync_committed_target(target)?;
    sync_parent_directory(target)
}

fn injected_commit_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "injected atomic file commit failure")
}

fn should_fail_before_commit(fault: AtomicWriteFault) -> bool {
    fault == AtomicWriteFault::Replace
}

fn should_fail_backup_sync(fault: AtomicWriteFault) -> bool {
    fault == AtomicWriteFault::BackupSync
}
