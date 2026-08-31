use std::io;
use std::path::Path;

#[cfg(unix)]
use std::fs;

#[cfg(windows)]
use std::path::PathBuf;

#[cfg(windows)]
use zircon_runtime::asset::project::ProjectPaths;

#[cfg(windows)]
use zircon_runtime_interface::project::session_lock::windows_project_session_mutex_name;

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
        const ERROR_ALREADY_EXISTS: i32 = 183;
        let mutex_name =
            session_mutex_name(project_root).map_err(|source| SessionGuardError::Io {
                operation: "resolve project session ownership lease path",
                path: project_root.to_path_buf(),
                source,
            })?;
        let mutex_name = mutex_name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
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
fn session_mutex_name(project_root: &Path) -> Result<String, io::Error> {
    let project_root = stable_project_root(project_root)?;
    Ok(windows_project_session_mutex_name(project_root))
}

#[cfg(all(test, windows))]
pub(in crate::core::recovery) fn session_mutex_name_for_test(project_root: &Path) -> String {
    session_mutex_name(project_root).expect("test project session path should resolve")
}

#[cfg(windows)]
fn stable_project_root(project_root: &Path) -> Result<PathBuf, io::Error> {
    ProjectPaths::resolve_path(project_root).map(|path| path.display_path().to_path_buf())
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

#[cfg(all(test, windows))]
mod tests {
    use std::fs;
    use std::io;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::super::SessionGuardError;
    use super::{SessionOwnershipLease, session_mutex_name};

    static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(1);
    const WINDOWS_ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

    #[test]
    fn session_mutex_name_resolves_directory_aliases_through_the_project_resolver() {
        let root = temporary_root("lease-alias");
        let physical = root.join("physical-project");
        let alias = root.join("project-alias");
        fs::create_dir_all(&physical).expect("create physical project root");
        create_directory_alias(&physical, &alias);

        assert_eq!(
            session_mutex_name(&physical).expect("resolve physical project root"),
            session_mutex_name(&alias).expect("resolve project directory alias")
        );

        fs::remove_dir_all(&root).expect("remove project alias fixture");
    }

    #[test]
    fn session_mutex_name_resolves_directory_symbolic_link_aliases_through_the_project_resolver() {
        let root = temporary_root("lease-symbolic-link");
        let physical = root.join("physical-project");
        let alias = root.join("project-alias");
        fs::create_dir_all(&physical).expect("create physical project root");
        if !create_directory_symbolic_link(&physical, &alias) {
            fs::remove_dir_all(&root).expect("remove project symbolic-link fixture");
            return;
        }

        assert_eq!(
            session_mutex_name(&physical).expect("resolve physical project root"),
            session_mutex_name(&alias).expect("resolve project directory symbolic link")
        );
        let lease = SessionOwnershipLease::acquire(
            &physical,
            &physical.join(".zircon").join("session.lock"),
        )
        .expect("acquire physical project lease");
        assert!(matches!(
            SessionOwnershipLease::acquire(&alias, &alias.join(".zircon").join("session.lock")),
            Err(SessionGuardError::AlreadyHeld { .. })
        ));
        drop(lease);

        fs::remove_dir_all(&root).expect("remove project symbolic-link fixture");
    }

    #[test]
    fn session_lease_rejects_a_directory_alias_while_the_physical_project_is_held() {
        let root = temporary_root("lease-physical-alias");
        let physical = root.join("physical-project");
        let alias = root.join("project-alias");
        fs::create_dir_all(&physical).expect("create physical project root");
        create_directory_alias(&physical, &alias);

        let lease = SessionOwnershipLease::acquire(
            &physical,
            &physical.join(".zircon").join("session.lock"),
        )
        .expect("acquire physical project lease");
        assert!(matches!(
            SessionOwnershipLease::acquire(&alias, &alias.join(".zircon").join("session.lock")),
            Err(SessionGuardError::AlreadyHeld { .. })
        ));
        drop(lease);

        fs::remove_dir_all(&root).expect("remove project alias fixture");
    }

    fn temporary_root(label: &str) -> std::path::PathBuf {
        std::env::current_dir()
            .expect("current directory should be available")
            .join("target")
            .join(format!(
                "zircon-editor-session-{label}-{}-{}",
                std::process::id(),
                NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
            ))
    }

    fn create_directory_alias(target: &std::path::Path, link: &std::path::Path) {
        let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
        let output = std::process::Command::new("cmd")
            .args(["/D", "/S", "/C"])
            .arg(command)
            .output()
            .expect("start mklink for project session alias fixture");
        assert!(
            output.status.success(),
            "create project session junction fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn create_directory_symbolic_link(target: &std::path::Path, link: &std::path::Path) -> bool {
        match std::os::windows::fs::symlink_dir(target, link) {
            Ok(()) => true,
            Err(error)
                if error.kind() == io::ErrorKind::PermissionDenied
                    || error.raw_os_error() == Some(WINDOWS_ERROR_PRIVILEGE_NOT_HELD) =>
            {
                false
            }
            Err(error) => panic!("create project session symbolic-link fixture failed: {error}"),
        }
    }
}
