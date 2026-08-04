use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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
    stage_atomic_write_with_fault(path, bytes, fault)?.commit()
}

pub(crate) fn stage_atomic_write(path: &Path, bytes: &[u8]) -> io::Result<PendingAtomicWrite> {
    stage_atomic_write_with_fault(path, bytes, AtomicWriteFault::None)
}

fn stage_atomic_write_with_fault(
    path: &Path,
    bytes: &[u8],
    fault: AtomicWriteFault,
) -> io::Result<PendingAtomicWrite> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    if let Some(parent) = parent {
        fs::create_dir_all(parent)?;
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
            return rename_staging(staging_path, path);
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

fn unique_sibling_path(directory: &Path, target: &Path, role: &str) -> PathBuf {
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data");
    loop {
        let id = NEXT_ATOMIC_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = directory.join(format!(
            ".{file_name}.zr-{role}-{}-{id}",
            std::process::id()
        ));
        if !candidate.exists() {
            return candidate;
        }
    }
}

fn rename_staging(staging_path: &Path, target: &Path) -> io::Result<()> {
    match fs::rename(staging_path, target) {
        Ok(()) => Ok(()),
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
        let _ = fs::remove_file(backup_path);
        return Err(error);
    }
    if should_fail_before_commit(fault) {
        let _ = fs::remove_file(staging_path);
        let _ = fs::remove_file(backup_path);
        return Err(injected_commit_error());
    }
    if let Err(error) = fs::rename(staging_path, target) {
        let _ = fs::remove_file(staging_path);
        let _ = fs::remove_file(backup_path);
        return Err(error);
    }
    let _ = fs::remove_file(backup_path);
    Ok(())
}

#[cfg(windows)]
fn commit_replace(
    target: &Path,
    staging_path: &Path,
    backup_path: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "Kernel32")]
    extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *mut c_void,
            reserved: *mut c_void,
        ) -> i32;
    }

    if should_fail_before_commit(fault) {
        let _ = fs::remove_file(staging_path);
        return Err(injected_commit_error());
    }
    if fault == AtomicWriteFault::ReplaceAfterBackup {
        fs::rename(target, backup_path)?;
        return handle_windows_replace_failure(
            target,
            staging_path,
            backup_path,
            injected_commit_error(),
        );
    }
    let target_wide = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let staging_wide = staging_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let backup_wide = backup_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        ReplaceFileW(
            target_wide.as_ptr(),
            staging_wide.as_ptr(),
            backup_wide.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        return handle_windows_replace_failure(
            target,
            staging_path,
            backup_path,
            io::Error::last_os_error(),
        );
    }
    // ReplaceFileW has committed both replacement and backup atomically. Cleanup is post-commit.
    let _ = fs::remove_file(backup_path);
    Ok(())
}

#[cfg(windows)]
fn handle_windows_replace_failure(
    target: &Path,
    staging_path: &Path,
    backup_path: &Path,
    replace_error: io::Error,
) -> io::Result<()> {
    let _ = fs::remove_file(staging_path);
    if backup_path.exists() && !target.exists() {
        return match fs::rename(backup_path, target) {
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
    if backup_path.exists() {
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
    Err(replace_error)
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

fn injected_commit_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "injected atomic file commit failure")
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

fn should_fail_before_commit(fault: AtomicWriteFault) -> bool {
    fault == AtomicWriteFault::Replace
}

pub(crate) fn recover_missing_target_from_backup(path: &Path) -> io::Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    recover_missing_target_from_backup_platform(path)
}

#[cfg(windows)]
fn recover_missing_target_from_backup_platform(path: &Path) -> io::Result<bool> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !directory.is_dir() {
        return Ok(false);
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data");
    let prefix = format!(".{file_name}.zr-backup-");
    let mut backups = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    backups.retain(|candidate| {
        candidate
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(&prefix))
    });
    backups.sort();
    match backups.as_slice() {
        [] => Ok(false),
        [backup] => {
            fs::rename(backup, path)?;
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

fn should_fail_backup_sync(fault: AtomicWriteFault) -> bool {
    fault == AtomicWriteFault::BackupSync
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{atomic_write_with_fault, AtomicWriteFault};

    #[test]
    fn atomic_write_replaces_existing_file_and_cleans_transaction_files() {
        let root = std::env::temp_dir().join(format!(
            "zircon_meta_atomic_commit_{}_{}",
            std::process::id(),
            super::NEXT_ATOMIC_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("hero.png.zmeta");
        fs::write(&path, "original").unwrap();

        atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::None).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "replacement");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_write_commit_failure_keeps_original_visible_and_cleans_staging_files() {
        let root = std::env::temp_dir().join(format!(
            "zircon_meta_atomic_restore_{}_{}",
            std::process::id(),
            super::NEXT_ATOMIC_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("hero.png.zmeta");
        fs::write(&path, "original").unwrap();

        let error =
            atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::Replace).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        assert!(path.is_file());
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_replace_failure_after_backup_restores_the_canonical_target() {
        let root = std::env::temp_dir().join(format!(
            "zircon_atomic_windows_restore_{}_{}",
            std::process::id(),
            super::NEXT_ATOMIC_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");
        fs::write(&path, "original").unwrap();

        let error =
            atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::ReplaceAfterBackup)
                .unwrap_err();

        assert!(error.to_string().contains("restored from backup"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_backup_sync_failure_cleans_staging_and_backup() {
        let root = std::env::temp_dir().join(format!(
            "zircon_meta_atomic_backup_sync_{}_{}",
            std::process::id(),
            super::NEXT_ATOMIC_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("hero.png.zmeta");
        fs::write(&path, "original").unwrap();

        let error = atomic_write_with_fault(&path, b"replacement", AtomicWriteFault::BackupSync)
            .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
