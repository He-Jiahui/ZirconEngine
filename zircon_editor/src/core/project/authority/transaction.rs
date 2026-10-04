use std::fs;
use std::path::Path;

use crate::core::project::ProjectAuthorityError;

/// Process-owned lease serializing the target while its project directory is published.
///
/// Windows uses a case-normalized target-qualified kernel mutex. Linux binds a target-qualified
/// abstract Unix socket, while other Unix targets conservatively flock the stable `/tmp`
/// directory. None of these leases depends on a replaceable project-parent inode, and live
/// ownership is released automatically on process death.
#[derive(Debug)]
pub(in crate::core::project) struct ProjectCreationLease {
    target: std::path::PathBuf,
    #[cfg(windows)]
    handle: isize,
    #[cfg(target_os = "linux")]
    _abstract_socket: std::os::unix::net::UnixDatagram,
    #[cfg(all(unix, not(target_os = "linux")))]
    global_lock_directory: fs::File,
    #[cfg(all(unix, not(target_os = "linux")))]
    _process_global_lease: ProcessGlobalCreationLease,
}

impl ProjectCreationLease {
    pub(in crate::core::project) fn acquire(target: &Path) -> Result<Self, ProjectAuthorityError> {
        #[cfg(windows)]
        {
            return Self::acquire_windows(target);
        }
        #[cfg(unix)]
        {
            return Self::acquire_unix(target);
        }
        #[cfg(not(any(windows, unix)))]
        {
            Err(ProjectAuthorityError::io(
                "acquire project creation lease",
                target,
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "project creation leases are not implemented for this platform",
                ),
            ))
        }
    }

    #[cfg(windows)]
    fn acquire_windows(target: &Path) -> Result<Self, ProjectAuthorityError> {
        const ERROR_ALREADY_EXISTS: i32 = 183;
        let mutex_name = windows_project_creation_mutex_name(target)?;
        let mutex_name = mutex_name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        // SAFETY: the name is NUL-terminated and the handle is non-inheritable. Clearing
        // last-error distinguishes a newly created mutex from an existing target lease.
        let handle = unsafe {
            SetLastError(0);
            CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr())
        };
        if handle == 0 {
            return Err(ProjectAuthorityError::io(
                "create project creation mutex",
                target,
                std::io::Error::last_os_error(),
            ));
        }
        if std::io::Error::last_os_error().raw_os_error() == Some(ERROR_ALREADY_EXISTS) {
            // SAFETY: CreateMutexW returned a valid handle for the pre-existing object.
            unsafe {
                CloseHandle(handle);
            }
            return Err(ProjectAuthorityError::TargetCreationLeaseHeld {
                path: target.to_path_buf(),
            });
        }
        Ok(Self {
            target: target.to_path_buf(),
            handle,
        })
    }

    #[cfg(unix)]
    fn acquire_unix(target: &Path) -> Result<Self, ProjectAuthorityError> {
        #[cfg(target_os = "linux")]
        {
            return Self::acquire_linux(target);
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self::acquire_global_unix(target)
        }
    }

    #[cfg(target_os = "linux")]
    fn acquire_linux(target: &Path) -> Result<Self, ProjectAuthorityError> {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::net::{SocketAddr, UnixDatagram};

        let identity = blake3::hash(target.as_os_str().as_bytes());
        let name = format!("zircon-engine-project-creation-{}", identity.to_hex());
        let address = SocketAddr::from_abstract_name(name.as_bytes()).map_err(|source| {
            ProjectAuthorityError::io("construct project creation lease identity", target, source)
        })?;
        let abstract_socket = UnixDatagram::bind_addr(&address).map_err(|source| {
            if source.kind() == std::io::ErrorKind::AddrInUse {
                ProjectAuthorityError::TargetCreationLeaseHeld {
                    path: target.to_path_buf(),
                }
            } else {
                ProjectAuthorityError::io("bind project creation lease", target, source)
            }
        })?;
        Ok(Self {
            target: target.to_path_buf(),
            _abstract_socket: abstract_socket,
        })
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    fn acquire_global_unix(target: &Path) -> Result<Self, ProjectAuthorityError> {
        use std::os::unix::io::AsRawFd;

        const LOCK_EX: i32 = 2;
        const LOCK_NB: i32 = 4;
        let process_global_lease = ProcessGlobalCreationLease::acquire().map_err(|()| {
            ProjectAuthorityError::TargetCreationLeaseHeld {
                path: target.to_path_buf(),
            }
        })?;
        let global_lock_directory = fs::File::open("/tmp").map_err(|source| {
            ProjectAuthorityError::io("open global project creation lease", "/tmp", source)
        })?;
        // SAFETY: the descriptor belongs to `global_lock_directory` and remains alive in the
        // lease. `/tmp` is a stable system directory rather than a replaceable app-owned file.
        if unsafe { flock(global_lock_directory.as_raw_fd(), LOCK_EX | LOCK_NB) } != 0 {
            let source = std::io::Error::last_os_error();
            if matches!(
                source.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::AlreadyExists
            ) {
                return Err(ProjectAuthorityError::TargetCreationLeaseHeld {
                    path: target.to_path_buf(),
                });
            }
            return Err(ProjectAuthorityError::io(
                "acquire global project creation lease",
                "/tmp",
                source,
            ));
        }
        Ok(Self {
            target: target.to_path_buf(),
            global_lock_directory,
            _process_global_lease: process_global_lease,
        })
    }

    pub(in crate::core::project) fn target(&self) -> &Path {
        &self.target
    }
}

#[cfg(windows)]
impl Drop for ProjectCreationLease {
    fn drop(&mut self) {
        // SAFETY: the handle is owned by this lease and is closed exactly once.
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
impl Drop for ProjectCreationLease {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        const LOCK_UN: i32 = 8;
        // SAFETY: the descriptor remains valid until this destructor returns.
        unsafe { flock(self.global_lock_directory.as_raw_fd(), LOCK_UN) };
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
#[derive(Debug)]
struct ProcessGlobalCreationLease;

#[cfg(all(unix, not(target_os = "linux")))]
static PROCESS_GLOBAL_PROJECT_CREATION_HELD: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(all(unix, not(target_os = "linux")))]
impl ProcessGlobalCreationLease {
    fn acquire() -> Result<Self, ()> {
        PROCESS_GLOBAL_PROJECT_CREATION_HELD
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .map(|_| Self)
            .map_err(|_| ())
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
impl Drop for ProcessGlobalCreationLease {
    fn drop(&mut self) {
        PROCESS_GLOBAL_PROJECT_CREATION_HELD.store(false, std::sync::atomic::Ordering::Release);
    }
}

#[cfg(windows)]
fn windows_project_creation_mutex_name(target: &Path) -> Result<String, ProjectAuthorityError> {
    use std::os::windows::ffi::OsStrExt;

    const LCMAP_UPPERCASE: u32 = 0x0000_0200;
    const INVARIANT_LOCALE_NAME: [u16; 1] = [0];

    let source = target.as_os_str().encode_wide().collect::<Vec<_>>();
    let source_len = i32::try_from(source.len()).map_err(|_| {
        ProjectAuthorityError::io(
            "normalize project creation lease identity",
            target,
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "project creation target exceeds the Windows locale-mapping limit",
            ),
        )
    })?;
    let required = unsafe {
        LCMapStringEx(
            INVARIANT_LOCALE_NAME.as_ptr(),
            LCMAP_UPPERCASE,
            source.as_ptr(),
            source_len,
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
            0,
        )
    };
    if required == 0 {
        return Err(ProjectAuthorityError::io(
            "normalize project creation lease identity",
            target,
            std::io::Error::last_os_error(),
        ));
    }
    let mut normalized = vec![0_u16; required as usize];
    let written = unsafe {
        LCMapStringEx(
            INVARIANT_LOCALE_NAME.as_ptr(),
            LCMAP_UPPERCASE,
            source.as_ptr(),
            source_len,
            normalized.as_mut_ptr(),
            required,
            std::ptr::null(),
            std::ptr::null_mut(),
            0,
        )
    };
    if written != required {
        return Err(ProjectAuthorityError::io(
            "normalize project creation lease identity",
            target,
            std::io::Error::last_os_error(),
        ));
    }
    let mut bytes = Vec::with_capacity(normalized.len() * 2);
    for unit in normalized {
        bytes.extend(unit.to_le_bytes());
    }
    Ok(format!(
        "Global\\ZirconEngineProjectCreation-{}",
        blake3::hash(&bytes).to_hex()
    ))
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
    fn LCMapStringEx(
        locale_name: *const u16,
        map_flags: u32,
        source: *const u16,
        source_length: i32,
        destination: *mut u16,
        destination_length: i32,
        version_information: *const std::ffi::c_void,
        reserved: *mut std::ffi::c_void,
        sort_handle: isize,
    ) -> i32;
}

#[cfg(unix)]
unsafe extern "C" {
    fn flock(file_descriptor: i32, operation: i32) -> i32;
}

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

fn finalize_empty_target_backup(
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

pub(in crate::core::project) fn finalize_published_project(
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
) -> Result<(), ProjectAuthorityError> {
    finalize_empty_target_backup(target, backup, replace_empty_target).map_err(|source| {
        ProjectAuthorityError::PublishedProjectFinalizationFailed {
            target: target.to_path_buf(),
            backup: backup.to_path_buf(),
            source: Box::new(source),
        }
    })
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
