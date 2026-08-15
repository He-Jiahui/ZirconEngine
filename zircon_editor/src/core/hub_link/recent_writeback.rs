use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::core::resource::io::atomic_write;
#[cfg(unix)]
use zircon_runtime_interface::hub_protocol::hub_recent_projects_lock_path;
use zircon_runtime_interface::hub_protocol::{
    hub_recent_projects_path, HubRecentProjectV1, HubRecentProjectsError, HubRecentProjectsV1,
};
use zircon_runtime_interface::project::ProjectManifestSummary;

/// Editor-owned reader and writer for the Hub/Editor shared recent-project registry.
///
/// The short write lease protects read-merge-write updates across both desktop processes. It is
/// unrelated to `SessionGuard`: project liveness remains scoped to a project, while this lease
/// only serializes a global history document.
pub(crate) fn load_recent_projects() -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    load_recent_projects_at(&hub_recent_projects_path())
}

pub(crate) fn record_recent_project(
    project_root: &Path,
    summary: ProjectManifestSummary,
    last_opened_unix_ms: u64,
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    record_recent_project_at(
        &hub_recent_projects_path(),
        project_root,
        summary,
        last_opened_unix_ms,
    )
}

pub(crate) fn forget_recent_project(
    project_root: &Path,
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    let display_path = ProjectPaths::resolve_path(project_root)
        .map(|resolved| resolved.display_path().to_path_buf())
        .unwrap_or_else(|_| project_root.to_path_buf());
    update_recent_projects_at(&hub_recent_projects_path(), |registry| {
        registry.remove(display_path);
    })
}

fn record_recent_project_at(
    registry_path: &Path,
    project_root: &Path,
    summary: ProjectManifestSummary,
    last_opened_unix_ms: u64,
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    let display_path = ProjectPaths::resolve_path(project_root)
        .map(|resolved| resolved.display_path().to_path_buf())
        .map_err(|source| HubRecentProjectsStoreError::Io {
            operation: "resolve project root for shared recent-project registry",
            path: project_root.to_path_buf(),
            source,
        })?;
    let project =
        HubRecentProjectV1::new(summary, display_path, last_opened_unix_ms).map_err(|source| {
            HubRecentProjectsStoreError::Contract {
                path: registry_path.to_path_buf(),
                source,
            }
        })?;
    update_recent_projects_at(registry_path, |registry| registry.record(project))
}

fn update_recent_projects_at(
    registry_path: &Path,
    update: impl FnOnce(&mut HubRecentProjectsV1),
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    let _lease = RecentProjectsWriteLease::acquire(registry_path)?;
    let mut registry = load_recent_projects_at(registry_path)?;
    update(&mut registry);
    registry
        .validate()
        .map_err(|source| HubRecentProjectsStoreError::Contract {
            path: registry_path.to_path_buf(),
            source,
        })?;
    write_recent_projects_at(registry_path, &registry)?;
    Ok(registry)
}

fn load_recent_projects_at(
    registry_path: &Path,
) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    if !registry_path.exists() {
        return Ok(HubRecentProjectsV1::default());
    }
    let bytes = fs::read(registry_path).map_err(|source| HubRecentProjectsStoreError::Io {
        operation: "read shared recent-project registry",
        path: registry_path.to_path_buf(),
        source,
    })?;
    let registry = serde_json::from_slice::<HubRecentProjectsV1>(&bytes).map_err(|source| {
        HubRecentProjectsStoreError::Decode {
            path: registry_path.to_path_buf(),
            source,
        }
    })?;
    registry
        .validate()
        .map_err(|source| HubRecentProjectsStoreError::Contract {
            path: registry_path.to_path_buf(),
            source,
        })?;
    Ok(registry)
}

fn write_recent_projects_at(
    registry_path: &Path,
    registry: &HubRecentProjectsV1,
) -> Result<(), HubRecentProjectsStoreError> {
    let bytes = serde_json::to_vec_pretty(registry).map_err(|source| {
        HubRecentProjectsStoreError::Encode {
            path: registry_path.to_path_buf(),
            source,
        }
    })?;
    atomic_write(registry_path, &bytes).map_err(|source| HubRecentProjectsStoreError::Io {
        operation: "atomically replace shared recent-project registry",
        path: registry_path.to_path_buf(),
        source,
    })
}

#[derive(Debug, Error)]
pub(crate) enum HubRecentProjectsStoreError {
    #[error("failed to {operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to decode shared recent-project registry `{path}`: {source}")]
    Decode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to encode shared recent-project registry `{path}`: {source}")]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("shared recent-project registry `{path}` violates the v1 contract: {source}")]
    Contract {
        path: PathBuf,
        #[source]
        source: HubRecentProjectsError,
    },
    #[error("shared recent-project registry writer lease is unsupported for `{path}`")]
    PlatformUnsupported { path: PathBuf },
}

#[derive(Debug)]
struct RecentProjectsWriteLease {
    #[cfg(windows)]
    handle: isize,
    #[cfg(unix)]
    lock_file: fs::File,
}

impl RecentProjectsWriteLease {
    fn acquire(registry_path: &Path) -> Result<Self, HubRecentProjectsStoreError> {
        #[cfg(windows)]
        {
            return Self::acquire_windows(registry_path);
        }
        #[cfg(unix)]
        {
            return Self::acquire_unix(registry_path);
        }
        #[cfg(not(any(windows, unix)))]
        {
            Err(HubRecentProjectsStoreError::PlatformUnsupported {
                path: registry_path.to_path_buf(),
            })
        }
    }

    #[cfg(windows)]
    fn acquire_windows(registry_path: &Path) -> Result<Self, HubRecentProjectsStoreError> {
        use zircon_runtime_interface::hub_protocol::windows_hub_recent_projects_mutex_name;

        const WAIT_OBJECT_0: u32 = 0;
        const WAIT_ABANDONED: u32 = 0x0000_0080;
        const INFINITE: u32 = 0xFFFF_FFFF;

        let mutex_name = windows_hub_recent_projects_mutex_name(registry_path)
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        // SAFETY: the name buffer is NUL-terminated for this synchronous call and the returned
        // handle is owned by the lease when non-zero.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr()) };
        if handle == 0 {
            return Err(HubRecentProjectsStoreError::Io {
                operation: "create shared recent-project writer lease",
                path: registry_path.to_path_buf(),
                source: io::Error::last_os_error(),
            });
        }
        // SAFETY: `handle` is a valid mutex handle returned above. An abandoned mutex grants this
        // caller ownership after a previous writer process terminated unexpectedly.
        match unsafe { WaitForSingleObject(handle, INFINITE) } {
            WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Self { handle }),
            _ => {
                // SAFETY: this local handle has not been transferred to a lease.
                unsafe {
                    CloseHandle(handle);
                }
                Err(HubRecentProjectsStoreError::Io {
                    operation: "acquire shared recent-project writer lease",
                    path: registry_path.to_path_buf(),
                    source: io::Error::last_os_error(),
                })
            }
        }
    }

    #[cfg(unix)]
    fn acquire_unix(registry_path: &Path) -> Result<Self, HubRecentProjectsStoreError> {
        use std::os::unix::io::AsRawFd;

        const LOCK_EX: i32 = 2;
        let lock_path = hub_recent_projects_lock_path(registry_path);
        let parent = lock_path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|source| HubRecentProjectsStoreError::Io {
            operation: "create shared recent-project registry directory",
            path: parent.to_path_buf(),
            source,
        })?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|source| HubRecentProjectsStoreError::Io {
                operation: "open shared recent-project writer lease",
                path: lock_path.clone(),
                source,
            })?;
        // SAFETY: the file descriptor belongs to `lock_file`, which remains owned by the lease
        // until the matching unlock in `Drop`.
        if unsafe { flock(lock_file.as_raw_fd(), LOCK_EX) } != 0 {
            return Err(HubRecentProjectsStoreError::Io {
                operation: "acquire shared recent-project writer lease",
                path: lock_path,
                source: io::Error::last_os_error(),
            });
        }
        Ok(Self { lock_file })
    }
}

#[cfg(windows)]
impl Drop for RecentProjectsWriteLease {
    fn drop(&mut self) {
        // SAFETY: this lease owns the mutex after `WaitForSingleObject` succeeded.
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}

#[cfg(unix)]
impl Drop for RecentProjectsWriteLease {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        const LOCK_UN: i32 = 8;
        // SAFETY: the descriptor remains valid until this destructor returns.
        unsafe {
            flock(self.lock_file.as_raw_fd(), LOCK_UN);
        }
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(
        attributes: *const std::ffi::c_void,
        initial_owner: i32,
        name: *const u16,
    ) -> isize;
    fn WaitForSingleObject(handle: isize, milliseconds: u32) -> u32;
    fn ReleaseMutex(handle: isize) -> i32;
    fn CloseHandle(handle: isize) -> i32;
}

#[cfg(unix)]
unsafe extern "C" {
    fn flock(file_descriptor: i32, operation: i32) -> i32;
}
