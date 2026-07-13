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
    BackupSync,
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_write_with_fault(path, bytes, AtomicWriteFault::None)
}

pub(crate) fn atomic_write_with_fault(
    path: &Path,
    bytes: &[u8],
    fault: AtomicWriteFault,
) -> io::Result<()> {
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

    if !path.exists() {
        if should_fail_before_commit(fault) {
            let _ = fs::remove_file(&staging_path);
            return Err(injected_commit_error());
        }
        return rename_staging(&staging_path, path);
    }

    let backup_path = unique_sibling_path(directory, path, "backup");
    commit_replace(path, &staging_path, &backup_path, fault)
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
        .unwrap_or("asset.zmeta");
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
                    "failed to preserve atomic meta backup: hard-link failed: {link_error}; copy failed: {copy_error}"
                ),
            ));
        }
    }
    let backup_sync = if should_fail_backup_sync(fault) {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic meta backup sync failure",
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
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let staging = staging_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let backup = backup_path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        ReplaceFileW(
            target.as_ptr(),
            staging.as_ptr(),
            backup.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        let error = io::Error::last_os_error();
        let _ = fs::remove_file(staging_path);
        if backup_path.exists() {
            return Err(io::Error::new(
                error.kind(),
                WindowsReplaceFailure {
                    os_error: error,
                    backup_path: backup_path.to_path_buf(),
                },
            ));
        }
        return Err(error);
    }
    // ReplaceFileW has committed both replacement and backup atomically. Cleanup is post-commit.
    let _ = fs::remove_file(backup_path);
    Ok(())
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
        "atomic meta replacement is unsupported on this platform",
    ))
}

fn injected_commit_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "injected atomic meta commit failure")
}

#[cfg(windows)]
#[derive(Debug)]
struct WindowsReplaceFailure {
    os_error: io::Error,
    backup_path: PathBuf,
}

#[cfg(windows)]
impl std::fmt::Display for WindowsReplaceFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "ReplaceFileW failed with OS error {:?} ({}) and retained backup at {}",
            self.os_error.raw_os_error(),
            self.os_error,
            self.backup_path.display()
        )
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
