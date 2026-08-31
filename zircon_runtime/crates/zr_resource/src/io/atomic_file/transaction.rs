use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::directory::{
    create_and_sync_parent_directories, sync_parent_directory, sync_parent_directory_with_fault,
};
use super::pathing::unique_sibling_path;
use super::platform;
use super::{AtomicWriteFault, PathEntry, path_entry};

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
pub struct PendingAtomicWrite {
    target: PathBuf,
    staging_path: PathBuf,
    fault: AtomicWriteFault,
}

impl PendingAtomicWrite {
    pub fn commit(self) -> io::Result<()> {
        let path = self.target.as_path();
        let staging_path = self.staging_path.as_path();

        match path_entry(path)? {
            PathEntry::Missing => {
                if should_fail_before_commit(self.fault) {
                    let _ = fs::remove_file(staging_path);
                    return Err(injected_commit_error());
                }
                return rename_staging(staging_path, path, self.fault);
            }
            PathEntry::File => {}
            PathEntry::Directory | PathEntry::Other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "atomic write target is not a regular file: {}",
                        path.display()
                    ),
                ));
            }
        }

        let directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let backup_path = unique_sibling_path(directory, path, "backup")?;
        commit_replace(path, staging_path, &backup_path, self.fault)
    }

    pub fn commit_new(self) -> io::Result<()> {
        let path = self.target.as_path();
        let staging_path = self.staging_path.as_path();
        match path_entry(path)? {
            PathEntry::Missing => {}
            PathEntry::File | PathEntry::Directory | PathEntry::Other => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("atomic write target already exists: {}", path.display()),
                ));
            }
        }
        publish_new_staging(staging_path, path, self.fault)
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
        let path = unique_sibling_path(directory, target, "staging")?;
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
fn publish_new_staging(
    staging_path: &Path,
    target: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    publish_new_staging_observed(staging_path, target, fault)
        .map_err(StagedPublicationError::into_io_error)
}

#[cfg(unix)]
fn publish_new_staging_observed(
    staging_path: &Path,
    target: &Path,
    fault: AtomicWriteFault,
) -> Result<(), StagedPublicationError> {
    if should_fail_before_commit(fault) {
        let _ = fs::remove_file(staging_path);
        return Err(StagedPublicationError::not_published(
            injected_commit_error(),
        ));
    }
    fs::hard_link(staging_path, target).map_err(StagedPublicationError::not_published)?;
    if let Err(error) = fs::remove_file(staging_path) {
        return Err(StagedPublicationError::may_have_published(io::Error::new(
            error.kind(),
            format!(
                "published {} but failed to remove atomic staging file {}: {error}",
                target.display(),
                staging_path.display()
            ),
        )));
    }
    sync_parent_directory_with_fault(target, fault)
        .map_err(StagedPublicationError::may_have_published)
}

#[cfg(windows)]
fn publish_new_staging(
    staging_path: &Path,
    target: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    publish_new_staging_observed(staging_path, target, fault)
        .map_err(StagedPublicationError::into_io_error)
}

#[cfg(windows)]
fn publish_new_staging_observed(
    staging_path: &Path,
    target: &Path,
    fault: AtomicWriteFault,
) -> Result<(), StagedPublicationError> {
    if should_fail_before_commit(fault) {
        let _ = fs::remove_file(staging_path);
        return Err(StagedPublicationError::not_published(
            injected_commit_error(),
        ));
    }
    if let Err(error) = platform::rename_staging(staging_path, target) {
        let _ = fs::remove_file(staging_path);
        return Err(StagedPublicationError::not_published(error));
    }
    sync_parent_directory_with_fault(target, fault)
        .map_err(StagedPublicationError::may_have_published)
}

#[cfg(not(any(unix, windows)))]
fn publish_new_staging(
    staging_path: &Path,
    _target: &Path,
    _fault: AtomicWriteFault,
) -> io::Result<()> {
    publish_new_staging_observed(staging_path, _target, _fault)
        .map_err(StagedPublicationError::into_io_error)
}

#[cfg(not(any(unix, windows)))]
fn publish_new_staging_observed(
    staging_path: &Path,
    _target: &Path,
    _fault: AtomicWriteFault,
) -> Result<(), StagedPublicationError> {
    let _ = fs::remove_file(staging_path);
    Err(StagedPublicationError::not_published(io::Error::new(
        io::ErrorKind::Unsupported,
        "atomic new-file publication is unsupported on this platform",
    )))
}

#[cfg(unix)]
fn commit_replace(
    target: &Path,
    staging_path: &Path,
    backup_path: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    if let Err(error) = prepare_backup(target, backup_path, fault) {
        let _ = fs::remove_file(staging_path);
        return Err(error);
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
    if let Err(error) = prepare_backup(target, backup_path, fault) {
        let _ = fs::remove_file(staging_path);
        return Err(error);
    }
    if should_fail_before_commit(fault) {
        let _ = fs::remove_file(staging_path);
        return Err(cleanup_precommit_backup_after_error(
            backup_path,
            injected_commit_error(),
        ));
    }
    if fault == AtomicWriteFault::ReplaceAfterBackup {
        if let Err(error) = fs::remove_file(target) {
            let _ = fs::remove_file(staging_path);
            return Err(cleanup_precommit_backup_after_error(backup_path, error));
        }
        let replace_error = match sync_parent_directory(target) {
            Ok(()) => injected_commit_error(),
            Err(error) => error,
        };
        return super::recovery::handle_windows_replace_failure(
            target,
            staging_path,
            backup_path,
            replace_error,
        );
    }
    if let Err(error) = platform::replace_existing_staged_file(staging_path, target) {
        return super::recovery::handle_windows_replace_failure(
            target,
            staging_path,
            backup_path,
            error,
        );
    }
    sync_windows_committed_file(target, fault)?;
    sync_parent_directory_with_fault(target, fault)?;
    // The backup was durably prepared before the atomic replacement. Cleanup is post-commit.
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
pub fn replace_staged_file(staging: &Path, target: &Path) -> io::Result<()> {
    match path_entry(target)? {
        PathEntry::File => platform::replace_existing_staged_file(staging, target)?,
        PathEntry::Missing => {
            return publish_new_staging(staging, target, AtomicWriteFault::None);
        }
        PathEntry::Directory | PathEntry::Other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "staged replacement target is not a regular file: {}",
                    target.display()
                ),
            ));
        }
    }
    platform::sync_committed_target(target)?;
    sync_parent_directory(target)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StagedPublicationState {
    NotPublished,
    MayHavePublished,
}

#[derive(Debug)]
pub(crate) struct StagedPublicationError {
    source: io::Error,
    state: StagedPublicationState,
}

impl StagedPublicationError {
    fn not_published(source: io::Error) -> Self {
        Self {
            source,
            state: StagedPublicationState::NotPublished,
        }
    }

    fn may_have_published(source: io::Error) -> Self {
        Self {
            source,
            state: StagedPublicationState::MayHavePublished,
        }
    }

    pub(crate) fn rollback_required(&self) -> bool {
        self.state == StagedPublicationState::MayHavePublished
    }

    pub(crate) fn into_io_error(self) -> io::Error {
        self.source
    }
}

pub(crate) fn publish_staged_file_for_transaction(
    staging: &Path,
    target: &Path,
    target_existed: bool,
) -> Result<(), StagedPublicationError> {
    if !target_existed {
        return publish_new_staging_observed(staging, target, AtomicWriteFault::None);
    }

    platform::replace_existing_staged_file(staging, target)
        .map_err(StagedPublicationError::may_have_published)?;
    platform::sync_committed_target(target).map_err(StagedPublicationError::may_have_published)?;
    sync_parent_directory(target).map_err(StagedPublicationError::may_have_published)
}

pub(super) fn copy_file_create_new(source: &Path, target: &Path) -> io::Result<()> {
    let mut source = File::open(source)?;
    let mut target_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    if let Err(error) = io::copy(&mut source, &mut target_file)
        .and_then(|_| target_file.flush())
        .and_then(|()| target_file.sync_all())
    {
        drop(target_file);
        return Err(cleanup_precommit_backup_after_error(target, error));
    }
    Ok(())
}

pub(super) fn create_backup_file_new(source: &Path, target: &Path) -> io::Result<()> {
    match fs::hard_link(source, target) {
        Ok(()) => Ok(()),
        Err(link_error) => copy_file_create_new(source, target).map_err(|copy_error| {
            io::Error::new(
                copy_error.kind(),
                format!(
                    "failed to preserve atomic file backup: hard-link failed: {link_error}; create-only copy failed: {copy_error}"
                ),
            )
        }),
    }
}

fn prepare_backup(source: &Path, backup: &Path, fault: AtomicWriteFault) -> io::Result<()> {
    create_backup_file_new(source, backup)?;
    let barrier = if should_fail_backup_sync(fault) {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic file backup sync failure",
        ))
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(backup)
            .and_then(|file| file.sync_all())
    }
    .and_then(|()| sync_parent_directory(backup));
    barrier.map_err(|error| cleanup_precommit_backup_after_error(backup, error))
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
