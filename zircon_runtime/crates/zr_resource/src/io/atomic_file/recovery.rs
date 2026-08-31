use std::io;
use std::path::Path;

use super::{PathEntry, path_entry};

#[cfg(windows)]
use super::directory::sync_parent_directory;
#[cfg(windows)]
use super::pathing::{is_atomic_write_transaction_path, target_file_name};
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::PathBuf;

pub fn recover_missing_target_from_backup(path: &Path) -> io::Result<bool> {
    match path_entry(path)? {
        PathEntry::Missing => recover_missing_target_from_backup_platform(path),
        PathEntry::File => Ok(false),
        PathEntry::Directory | PathEntry::Other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "atomic recovery target is not a regular file: {}",
                path.display()
            ),
        )),
    }
}

#[cfg(windows)]
fn recover_missing_target_from_backup_platform(path: &Path) -> io::Result<bool> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    match path_entry(directory)? {
        PathEntry::Missing => return Ok(false),
        PathEntry::Directory => {}
        PathEntry::File | PathEntry::Other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "atomic recovery owner is not a real directory: {}",
                    directory.display()
                ),
            ));
        }
    }
    let prefix = format!(".{}.zr-backup-", target_file_name(path));
    let mut backups = Vec::new();
    for entry in fs::read_dir(directory)? {
        let candidate = entry?.path();
        let reserved_backup = candidate
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(&prefix))
            && is_atomic_write_transaction_path(&candidate);
        if !reserved_backup {
            continue;
        }
        match path_entry(&candidate)? {
            PathEntry::File => backups.push(candidate),
            PathEntry::Missing => {}
            PathEntry::Directory | PathEntry::Other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "atomic recovery backup is not a regular file: {}",
                        candidate.display()
                    ),
                ));
            }
        }
    }
    backups.sort();
    match backups.as_slice() {
        [] => Ok(false),
        [backup] => {
            super::platform::rename_staging(backup, path)?;
            super::platform::sync_committed_target(path)?;
            sync_parent_directory(path)?;
            Ok(true)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "cannot recover missing atomic target {} from {} backup candidates",
                path.display(),
                backups.len()
            ),
        )),
    }
}

#[cfg(not(windows))]
fn recover_missing_target_from_backup_platform(_path: &Path) -> io::Result<bool> {
    Ok(false)
}

#[cfg(windows)]
pub(super) fn handle_windows_replace_failure(
    target: &Path,
    staging_path: &Path,
    backup_path: &Path,
    replace_error: io::Error,
) -> io::Result<()> {
    let _ = fs::remove_file(staging_path);
    let backup = path_entry(backup_path)?;
    let target_entry = path_entry(target)?;
    if backup == PathEntry::File && target_entry == PathEntry::Missing {
        return match super::platform::rename_staging(backup_path, target) {
            Ok(()) => Err(io::Error::new(
                replace_error.kind(),
                WindowsReplaceFailure {
                    os_error: replace_error,
                    recovery: WindowsReplaceRecovery::Restored,
                },
            )),
            Err(restore_error) => Err(io::Error::new(
                replace_error.kind(),
                WindowsReplaceFailure {
                    os_error: replace_error,
                    recovery: WindowsReplaceRecovery::RetainedBackup {
                        backup_path: backup_path.to_path_buf(),
                        restore_error: Some(restore_error),
                    },
                },
            )),
        };
    }
    if backup == PathEntry::File {
        return Err(io::Error::new(
            replace_error.kind(),
            WindowsReplaceFailure {
                os_error: replace_error,
                recovery: WindowsReplaceRecovery::RetainedBackup {
                    backup_path: backup_path.to_path_buf(),
                    restore_error: None,
                },
            },
        ));
    }
    match (backup, target_entry) {
        (PathEntry::Missing, PathEntry::Missing | PathEntry::File) => Err(replace_error),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "atomic replacement recovery found a non-file path; original error: {replace_error}"
            ),
        )),
    }
}

#[cfg(windows)]
#[derive(Debug)]
struct WindowsReplaceFailure {
    os_error: io::Error,
    recovery: WindowsReplaceRecovery,
}

#[cfg(windows)]
#[derive(Debug)]
enum WindowsReplaceRecovery {
    Restored,
    RetainedBackup {
        backup_path: PathBuf,
        restore_error: Option<io::Error>,
    },
}

#[cfg(windows)]
impl std::fmt::Display for WindowsReplaceFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "ReplaceFileW failed with OS error {:?} ({})",
            self.os_error.raw_os_error(),
            self.os_error
        )?;
        match &self.recovery {
            WindowsReplaceRecovery::Restored => {
                write!(formatter, "; the canonical target was restored from backup")
            }
            WindowsReplaceRecovery::RetainedBackup {
                backup_path,
                restore_error,
            } => {
                write!(formatter, "; retained backup at {}", backup_path.display())?;
                if let Some(restore_error) = restore_error {
                    write!(formatter, "; restore failed: {restore_error}")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(windows)]
impl std::error::Error for WindowsReplaceFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.os_error)
    }
}
