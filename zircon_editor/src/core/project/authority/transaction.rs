use std::fs;
use std::path::Path;

use crate::core::project::ProjectAuthorityError;

pub(in crate::core::project) fn commit_staged_directory<R>(
    staging: &Path,
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
    mut rename: R,
) -> Result<(), ProjectAuthorityError>
where
    R: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    if replace_empty_target {
        rename(target, backup).map_err(|source| {
            ProjectAuthorityError::io("stage empty target rollback", target, source)
        })?;
        match directory_is_empty(backup) {
            Ok(true) => {}
            Ok(false) => {
                let commit_source = std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "project target became non-empty during creation",
                );
                rename(backup, target).map_err(|restore_source| {
                    ProjectAuthorityError::CommitRollbackFailed {
                        target: target.to_path_buf(),
                        backup: backup.to_path_buf(),
                        commit_source,
                        restore_source,
                    }
                })?;
                return Err(ProjectAuthorityError::TargetNotEmpty {
                    path: target.to_path_buf(),
                });
            }
            Err(commit_source) => {
                if let Err(restore_source) = rename(backup, target) {
                    return Err(ProjectAuthorityError::CommitRollbackFailed {
                        target: target.to_path_buf(),
                        backup: backup.to_path_buf(),
                        commit_source,
                        restore_source,
                    });
                }
                return Err(ProjectAuthorityError::io(
                    "recheck empty project target before commit",
                    target,
                    commit_source,
                ));
            }
        }
    }

    if let Err(commit_source) = rename(staging, target) {
        if replace_empty_target {
            if let Err(restore_source) = rename(backup, target) {
                return Err(ProjectAuthorityError::CommitRollbackFailed {
                    target: target.to_path_buf(),
                    backup: backup.to_path_buf(),
                    commit_source,
                    restore_source,
                });
            }
        }
        return Err(ProjectAuthorityError::io(
            "commit project template",
            target,
            commit_source,
        ));
    }

    Ok(())
}

fn directory_is_empty(path: &Path) -> std::io::Result<bool> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().transpose()?.is_none())
}

pub(in crate::core::project) fn rollback_committed_project<R>(
    staging: &Path,
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
    mut rename: R,
) -> Result<(), ProjectAuthorityError>
where
    R: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    // Return the newly published directory to the transaction path so the caller's ordinary
    // error cleanup removes only this creation attempt.
    rename(target, staging).map_err(|source| ProjectAuthorityError::PostCommitRollbackFailed {
        from: target.to_path_buf(),
        to: staging.to_path_buf(),
        backup: replace_empty_target.then(|| backup.to_path_buf()),
        source,
    })?;
    if replace_empty_target {
        rename(backup, target).map_err(|source| {
            ProjectAuthorityError::PostCommitRollbackFailed {
                from: backup.to_path_buf(),
                to: target.to_path_buf(),
                backup: Some(backup.to_path_buf()),
                source,
            }
        })?;
    }
    Ok(())
}

pub(in crate::core::project) fn finalize_empty_target_backup(
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
) -> Result<(), ProjectAuthorityError> {
    if !replace_empty_target {
        return Ok(());
    }

    match fs::remove_dir(backup) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => match directory_is_empty(backup) {
            Ok(false) => Err(ProjectAuthorityError::TargetNotEmpty {
                path: target.to_path_buf(),
            }),
            Err(inspect_source) if inspect_source.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Ok(true) | Err(_) => Err(ProjectAuthorityError::io(
                "finalize empty project target backup",
                backup,
                source,
            )),
        },
    }
}

pub(in crate::core::project) fn cleanup_failed_transaction_staging(
    staging: &Path,
    preserve_rollback_artifacts: bool,
    staging_created: bool,
) {
    // A failed staging creation has no ownership of a pre-existing path, so cleanup may only
    // remove a directory created by this transaction and not retained for rollback recovery.
    if staging_created && !preserve_rollback_artifacts {
        remove_transaction_path(staging);
    }
}

fn remove_transaction_path(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}
