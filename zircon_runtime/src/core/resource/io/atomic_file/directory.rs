use std::fs;
use std::io;
use std::path::Path;

use super::platform;
use super::AtomicWriteFault;

pub(super) fn create_and_sync_parent_directories(
    parent: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    let mut created = Vec::new();
    let mut current = Some(parent);
    while let Some(directory) = current {
        if directory.exists() {
            break;
        }
        created.push(directory.to_path_buf());
        current = directory
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty());
    }

    for directory in created.iter().rev() {
        let created_here = match fs::create_dir(directory) {
            Ok(()) => true,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => false,
            Err(error) => return Err(error),
        };
        sync_new_directory(directory, created_here, fault)?;
    }
    Ok(())
}

pub(super) fn sync_new_directory(
    directory: &Path,
    created_here: bool,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    let barrier = sync_created_directory_with_fault(directory, fault)
        .and_then(|()| sync_parent_directory(directory));
    match barrier {
        Ok(()) => Ok(()),
        Err(error) if created_here => {
            Err(cleanup_unconfirmed_directory_after_error(directory, error))
        }
        Err(error) => Err(error),
    }
}

fn cleanup_unconfirmed_directory_after_error(directory: &Path, error: io::Error) -> io::Error {
    match fs::remove_dir(directory).and_then(|()| sync_parent_directory(directory)) {
        Ok(()) => error,
        Err(cleanup_error) => io::Error::new(
            error.kind(),
            format!(
                "{error}; failed to durably remove unconfirmed directory {}: {cleanup_error}",
                directory.display()
            ),
        ),
    }
}

fn sync_created_directory_with_fault(path: &Path, fault: AtomicWriteFault) -> io::Result<()> {
    if fault == AtomicWriteFault::CreatedDirectorySync {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic file created-directory sync failure",
        ));
    }
    platform::sync_directory(path)
}

/// Synchronizes the directory entry containing a committed file where the platform supports it.
///
/// Atomic-write consumers and durable removals share this owner so one caller cannot accidentally
/// provide stronger crash semantics than the rest of the engine.
pub(crate) fn sync_parent_directory(path: &Path) -> io::Result<()> {
    sync_parent_directory_with_fault(path, AtomicWriteFault::None)
}

pub(super) fn sync_parent_directory_with_fault(
    path: &Path,
    fault: AtomicWriteFault,
) -> io::Result<()> {
    if fault == AtomicWriteFault::ParentSync {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "injected atomic file parent-directory sync failure",
        ));
    }
    platform::sync_parent_directory_entry(path)
}

/// Creates a target's missing parent directories and publishes their directory entries.
pub(crate) fn ensure_parent_directories(path: &Path) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        create_and_sync_parent_directories(parent, AtomicWriteFault::None)?;
    }
    Ok(())
}
