use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::{
    SESSION_LOCK_FILE_NAME, SessionGuardError, SessionLockDurability, SessionLockRecord,
    encode_record,
};

pub(super) fn create_lock(
    path: &Path,
    record: &SessionLockRecord,
) -> Result<SessionLockDurability, SessionGuardError> {
    let temporary_path = stage_lock(path, record)?;
    match fs::hard_link(&temporary_path, path) {
        Ok(()) => {
            let _ = fs::remove_file(&temporary_path);
            Ok(durability_after_publish(path))
        }
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(&temporary_path);
            Err(SessionGuardError::AlreadyHeld {
                path: path.to_path_buf(),
                record: super::read_lock(path)?,
            })
        }
        Err(source) => {
            let _ = fs::remove_file(&temporary_path);
            Err(SessionGuardError::Io {
                operation: "publish session lock",
                path: path.to_path_buf(),
                source,
            })
        }
    }
}

pub(super) fn replace_lock(
    path: &Path,
    record: &SessionLockRecord,
) -> Result<SessionLockDurability, SessionGuardError> {
    let temporary_path = stage_lock(path, record)?;
    if let Err(source) = atomic_replace_existing(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(SessionGuardError::Io {
            operation: "replace session lock",
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(durability_after_publish(path))
}

pub(super) fn remove_lock(path: &Path) -> Result<SessionLockDurability, SessionGuardError> {
    fs::remove_file(path).map_err(|source| SessionGuardError::Io {
        operation: "remove session lock during normal shutdown",
        path: path.to_path_buf(),
        source,
    })?;
    Ok(durability_after_publish(path))
}

fn stage_lock(path: &Path, record: &SessionLockRecord) -> Result<PathBuf, SessionGuardError> {
    let directory = path
        .parent()
        .expect("session lock is always below the project .zircon directory");
    fs::create_dir_all(directory).map_err(|source| SessionGuardError::Io {
        operation: "create session lock directory",
        path: directory.to_path_buf(),
        source,
    })?;
    let temporary_path = directory.join(format!(
        ".{SESSION_LOCK_FILE_NAME}.{}.tmp",
        record.instance_id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .map_err(|source| SessionGuardError::Io {
            operation: "create temporary session lock",
            path: temporary_path.clone(),
            source,
        })?;
    if let Err(error) = write_record(&mut file, &temporary_path, record) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    drop(file);
    Ok(temporary_path)
}

fn write_record(
    file: &mut fs::File,
    path: &Path,
    record: &SessionLockRecord,
) -> Result<(), SessionGuardError> {
    file.write_all(encode_record(record).as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| SessionGuardError::Io {
            operation: "write session lock",
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(not(windows))]
fn durability_after_publish(path: &Path) -> SessionLockDurability {
    match sync_parent_directory(path) {
        Ok(()) => SessionLockDurability::Published,
        Err(_) => SessionLockDurability::PublishedWithDurabilityUncertainty,
    }
}

#[cfg(windows)]
fn durability_after_publish(_path: &Path) -> SessionLockDurability {
    // ReplaceFileW atomically publishes the fully flushed staging file, but this layer has no
    // parent-directory fsync equivalent on Windows. The guard is live, not durable-confirmed.
    SessionLockDurability::PublishedWithDurabilityUncertainty
}

#[cfg(windows)]
fn atomic_replace_existing(staging: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;

    if !destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "selected session lock no longer exists",
        ));
    }
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let staging = staging
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // SAFETY: both paths are NUL-terminated and live through the call. No backup
    // path is requested because a failed replacement must leave the residual lock intact.
    let replaced = unsafe {
        ReplaceFileW(
            destination.as_ptr(),
            staging.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn atomic_replace_existing(staging: &Path, destination: &Path) -> io::Result<()> {
    if !destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "selected session lock no longer exists",
        ));
    }
    fs::rename(staging, destination)
}

#[cfg(not(windows))]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}
