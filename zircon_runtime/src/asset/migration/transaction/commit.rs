//! Live-target commit, in-process rollback, and artifact cleanup.

use std::fs;
use std::io;
use std::path::Path;

use super::journal::record_document_state;
use super::schema::{CommitFault, JournalPhase, JournalState};
use super::stage::StagedDocument;
use super::{record_phase, remove_if_exists, transaction_error};
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};

pub(super) fn commit_document(
    document: &mut StagedDocument,
    fault: CommitFault,
    document_index: usize,
) -> io::Result<()> {
    #[cfg(not(test))]
    let _ = (fault, document_index);
    replace_synced_file(&document.staging, &document.target)?;
    document.committed = true;
    #[cfg(test)]
    if matches!(fault, CommitFault::CrashAfterTargetReplace(index) if index == document_index) {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "injected interruption after target replacement",
        ));
    }
    if let Some(retired) = &document.retired_path {
        fs::remove_file(retired)?;
        sync_parent_directory(retired)?;
        #[cfg(test)]
        if matches!(fault, CommitFault::CrashAfterRetiredDelete(index) if index == document_index) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "injected interruption after retired sidecar deletion",
            ));
        }
    }
    Ok(())
}

fn replace_synced_file(staging: &Path, target: &Path) -> io::Result<()> {
    if target.exists() {
        replace_existing_synced_file(staging, target)?;
    } else {
        replace_missing_synced_file(staging, target)?;
    }
    sync_parent_directory(target)
}

#[cfg(not(windows))]
fn replace_existing_synced_file(staging: &Path, target: &Path) -> io::Result<()> {
    fs::rename(staging, target)
}

#[cfg(not(windows))]
fn replace_missing_synced_file(staging: &Path, target: &Path) -> io::Result<()> {
    fs::rename(staging, target)
}

#[cfg(windows)]
fn replace_existing_synced_file(staging: &Path, target: &Path) -> io::Result<()> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    const REPLACEFILE_WRITE_THROUGH: u32 = 0x0000_0001;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *mut c_void,
            reserved: *mut c_void,
        ) -> i32;
    }

    let target_wide = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let staging_wide = staging
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        ReplaceFileW(
            target_wide.as_ptr(),
            staging_wide.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(windows)]
fn replace_missing_synced_file(staging: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(
            existing_file_name: *const u16,
            new_file_name: *const u16,
            flags: u32,
        ) -> i32;
    }

    let staging_wide = staging
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let target_wide = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let moved = unsafe {
        MoveFileExW(
            staging_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

pub(super) fn rollback_and_cleanup(
    journal: &Path,
    staged: &mut [StagedDocument],
    committed_end: usize,
    fault: CommitFault,
) -> Result<(), AssetMigrationError> {
    let mut first_error = None;
    for (index, document) in staged[..committed_end].iter_mut().enumerate().rev() {
        if !document.committed {
            continue;
        }
        record_document_state(journal, index, JournalState::RollingBack)?;
        let mut restored = true;
        if should_fail_restore(fault, index) {
            first_error.get_or_insert_with(|| {
                io::Error::new(io::ErrorKind::Other, "injected migration restore failure")
            });
            continue;
        }
        if let Some(backup) = document.backup.as_ref() {
            match replace_synced_file(backup, &document.target) {
                Ok(()) => {}
                Err(error) => {
                    first_error.get_or_insert(error);
                    restored = false;
                }
            }
        } else if let Err(error) = fs::remove_file(&document.target) {
            if error.kind() != io::ErrorKind::NotFound {
                first_error.get_or_insert(error);
                restored = false;
            }
        }
        if let (Some(retired), Some(backup)) = (
            document.retired_path.as_ref(),
            document.retired_backup.as_ref(),
        ) {
            match replace_synced_file(backup, retired) {
                Ok(()) => {}
                Err(error) => {
                    first_error.get_or_insert(error);
                    restored = false;
                }
            }
        }
        if restored {
            document.committed = false;
        }
    }
    if let Some(error) = first_error {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            error,
        ));
    }
    record_phase(journal, JournalPhase::RollbackCompleted)?;
    #[cfg(test)]
    if matches!(fault, CommitFault::CrashAfterRollbackCompleted { .. }) {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            io::Error::new(
                io::ErrorKind::Interrupted,
                "injected rollback cleanup interruption",
            ),
        ));
    }
    record_phase(journal, JournalPhase::CleanupRollback)?;
    cleanup_rollback_artifacts(journal, staged, fault)
}

fn cleanup_rollback_artifacts(
    journal: &Path,
    staged: &[StagedDocument],
    fault: CommitFault,
) -> Result<(), AssetMigrationError> {
    for document in staged {
        for artifact in [
            Some(&document.staging),
            document.backup.as_ref(),
            document.retired_backup.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            remove_if_exists(artifact).map_err(|source| {
                transaction_error(
                    AssetMigrationTransactionPhase::Rollback,
                    artifact.to_path_buf(),
                    source,
                )
            })?;
        }
    }
    #[cfg(test)]
    if matches!(fault, CommitFault::FailRollbackJournalDelete { .. }) {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected rollback journal deletion failure",
            ),
        ));
    }
    remove_if_exists(journal).map_err(|source| {
        transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            source,
        )
    })
}

pub(super) fn cleanup_committed_artifacts(
    journal: &Path,
    staged: &[StagedDocument],
) -> Result<(), AssetMigrationError> {
    for document in staged {
        for artifact in [
            Some(&document.staging),
            document.backup.as_ref(),
            document.retired_backup.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            remove_if_exists(artifact).map_err(|source| {
                transaction_error(
                    AssetMigrationTransactionPhase::Commit,
                    artifact.to_path_buf(),
                    source,
                )
            })?;
        }
    }
    fs::remove_file(journal).map_err(|source| {
        transaction_error(
            AssetMigrationTransactionPhase::Commit,
            journal.to_path_buf(),
            source,
        )
    })
}

pub(super) fn should_fail(fault: CommitFault, index: usize) -> bool {
    #[cfg(not(test))]
    let _ = index;
    match fault {
        CommitFault::Never => false,
        #[cfg(test)]
        CommitFault::CrashAfter(_) => false,
        #[cfg(test)]
        CommitFault::CrashAfterAllCommitted | CommitFault::CrashAfterCleanup => false,
        #[cfg(test)]
        CommitFault::CrashAfterRollbackCompleted { commit_index }
        | CommitFault::FailRollbackJournalDelete { commit_index } => commit_index == index,
        #[cfg(test)]
        CommitFault::At(fault_index) => fault_index == index,
        #[cfg(test)]
        CommitFault::AtWithRestoreFailure { commit_index, .. } => commit_index == index,
        #[cfg(test)]
        CommitFault::FailStageWrite(_)
        | CommitFault::FailBackupCopy(_)
        | CommitFault::FailRetiredBackupSync(_)
        | CommitFault::CrashAfterStaging(_)
        | CommitFault::CrashAfterTargetReplace(_)
        | CommitFault::CrashAfterRetiredDelete(_) => false,
    }
}

fn should_fail_restore(fault: CommitFault, index: usize) -> bool {
    #[cfg(not(test))]
    let _ = index;
    match fault {
        #[cfg(test)]
        CommitFault::AtWithRestoreFailure { restore_index, .. } => restore_index == index,
        _ => false,
    }
}
