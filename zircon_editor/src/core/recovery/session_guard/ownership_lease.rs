use std::fs;
use std::io;
use std::path::Path;

#[cfg(windows)]
use std::path::PathBuf;

use super::{SessionGuardError, read_lock, session_lock_directory};

/// An operating-system lease serializes every read/replace/remove operation for one project.
/// It is deliberately separate from the persisted record: process death releases the lease,
/// leaving `session.lock` available for explicit residual recovery.
#[derive(Debug)]
pub(super) struct SessionOwnershipLease {
    #[cfg(windows)]
    handle: isize,
    #[cfg(unix)]
    directory: fs::File,
}

impl SessionOwnershipLease {
    pub(super) fn acquire(
        project_root: &Path,
        lock_path: &Path,
    ) -> Result<Self, SessionGuardError> {
        #[cfg(windows)]
        {
            return Self::acquire_windows(project_root, lock_path);
        }
        #[cfg(unix)]
        {
            return Self::acquire_unix(project_root, lock_path);
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = (project_root, lock_path);
            Err(SessionGuardError::PlatformUnsupported)
        }
    }

    #[cfg(windows)]
    fn acquire_windows(project_root: &Path, lock_path: &Path) -> Result<Self, SessionGuardError> {
        use std::os::windows::ffi::OsStrExt;

        const ERROR_ALREADY_EXISTS: i32 = 183;
        let mutex_name = session_mutex_name(project_root);
        let mutex_name = mutex_name.encode_wide().chain(Some(0)).collect::<Vec<_>>();
        // SAFETY: the name is NUL-terminated and the null security descriptor requests a
        // non-inheritable handle with the process default access policy. Clearing last-error
        // lets this call distinguish a newly created mutex from an existing one reliably.
        let handle = unsafe {
            SetLastError(0);
            CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr())
        };
        if handle == 0 {
            return Err(SessionGuardError::Io {
                operation: "create project session ownership lease",
                path: lock_path.to_path_buf(),
                source: io::Error::last_os_error(),
            });
        }
        if io::Error::last_os_error().raw_os_error() == Some(ERROR_ALREADY_EXISTS) {
            // SAFETY: CreateMutexW returned a valid handle for the pre-existing object.
            unsafe {
                CloseHandle(handle);
            }
            return Err(already_held(lock_path)?);
        }
        Ok(Self { handle })
    }

    #[cfg(unix)]
    fn acquire_unix(project_root: &Path, lock_path: &Path) -> Result<Self, SessionGuardError> {
        use std::os::unix::io::AsRawFd;

        const LOCK_EX: i32 = 2;
        const LOCK_NB: i32 = 4;
        let directory = session_lock_directory(project_root);
        fs::create_dir_all(&directory).map_err(|source| SessionGuardError::Io {
            operation: "create project session directory for ownership lease",
            path: directory.clone(),
            source,
        })?;
        let directory_handle =
            fs::File::open(&directory).map_err(|source| SessionGuardError::Io {
                operation: "open project session directory for ownership lease",
                path: directory.clone(),
                source,
            })?;
        // SAFETY: the file descriptor belongs to `directory_handle` and is retained in the
        // returned lease until the matching unlock/drop path.
        let locked = unsafe { flock(directory_handle.as_raw_fd(), LOCK_EX | LOCK_NB) };
        if locked != 0 {
            let source = io::Error::last_os_error();
            if matches!(
                source.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::AlreadyExists
            ) {
                return Err(already_held(lock_path)?);
            }
            return Err(SessionGuardError::Io {
                operation: "acquire project session ownership lease",
                path: lock_path.to_path_buf(),
                source,
            });
        }
        Ok(Self {
            directory: directory_handle,
        })
    }
}

#[cfg(windows)]
impl Drop for SessionOwnershipLease {
    fn drop(&mut self) {
        // SAFETY: `handle` is owned by this lease and is closed exactly once on drop.
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[cfg(unix)]
impl Drop for SessionOwnershipLease {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        const LOCK_UN: i32 = 8;
        // SAFETY: the descriptor remains valid until this destructor returns.
        unsafe {
            flock(self.directory.as_raw_fd(), LOCK_UN);
        }
    }
}

fn already_held(lock_path: &Path) -> Result<SessionGuardError, SessionGuardError> {
    Ok(SessionGuardError::AlreadyHeld {
        path: lock_path.to_path_buf(),
        record: read_lock(lock_path)?,
    })
}

#[cfg(windows)]
fn session_mutex_name(project_root: &Path) -> String {
    use std::os::windows::ffi::OsStrExt;

    let project_root = stable_project_root(project_root);
    let mut bytes = Vec::new();
    for unit in project_root.as_os_str().encode_wide() {
        bytes.extend(unit.to_le_bytes());
    }
    format!(
        "Global\\ZirconEngineProjectSession-{}",
        blake3::hash(&bytes).to_hex()
    )
}

#[cfg(all(test, windows))]
pub(super) fn session_mutex_name_for_test(project_root: &Path) -> String {
    session_mutex_name(project_root)
}

#[cfg(windows)]
fn stable_project_root(project_root: &Path) -> PathBuf {
    let absolute = if project_root.is_absolute() {
        project_root.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(project_root))
            .unwrap_or_else(|_| project_root.to_path_buf())
    };
    fs::canonicalize(&absolute).unwrap_or(absolute)
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(
        attributes: *const std::ffi::c_void,
        initial_owner: i32,
        name: *const u16,
    ) -> isize;
    fn CloseHandle(handle: isize) -> i32;
    fn SetLastError(error: u32);
}

#[cfg(unix)]
unsafe extern "C" {
    fn flock(file_descriptor: i32, operation: i32) -> i32;
}
