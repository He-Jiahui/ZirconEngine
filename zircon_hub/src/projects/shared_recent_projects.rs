use std::fs;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
#[cfg(unix)]
use zircon_runtime_interface::hub_protocol::hub_recent_projects_lock_path;
use zircon_runtime_interface::hub_protocol::{
    hub_recent_project_path_key, HubRecentProjectV1, HubRecentProjectsError, HubRecentProjectsV1,
};

use super::metadata::normalize_project_root;
use super::RecentProject;

/// Loads the strict, versioned Hub/Editor shared recent-project registry.
pub fn load_shared_recent_projects(
    registry_path: impl AsRef<Path>,
) -> Result<Vec<RecentProject>, SharedRecentProjectsError> {
    let registry = load_registry(registry_path.as_ref())?;
    Ok(registry
        .projects
        .into_iter()
        .map(recent_project_from_shared)
        .collect())
}

/// Normalizes an in-memory Hub history through the same v1 DTO merge rules as the shared file.
pub fn merge_recent_project_entries<I, J>(
    left: I,
    right: J,
) -> Result<Vec<RecentProject>, SharedRecentProjectsError>
where
    I: IntoIterator<Item = RecentProject>,
    J: IntoIterator<Item = RecentProject>,
{
    let contract_path = Path::new("HubConfig.recent_projects");
    let projects = left
        .into_iter()
        .chain(right)
        .map(|project| shared_project_from_recent(contract_path, project))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HubRecentProjectsV1::new(projects)
        .projects
        .into_iter()
        .map(recent_project_from_shared)
        .collect())
}

/// Reconciles Hub changes against the current shared Hub/Editor registry atomically.
///
/// `previous_hub_projects` is the last snapshot synchronized into this Hub process. Comparing it
/// with `hub_projects` lets external Editor changes win unless the Hub changed that project after
/// the snapshot, so stale Hub memory cannot revive an Editor-side removal.
pub fn reconcile_shared_recent_projects(
    registry_path: impl AsRef<Path>,
    previous_hub_projects: &[RecentProject],
    hub_projects: &[RecentProject],
) -> Result<Vec<RecentProject>, SharedRecentProjectsError> {
    let registry_path = registry_path.as_ref();
    let _lease = SharedRecentProjectsWriteLease::acquire(registry_path)?;
    let mut registry = load_registry(registry_path)?;
    let hub_projects = hub_projects
        .iter()
        .cloned()
        .map(|project| shared_project_from_recent(registry_path, project))
        .collect::<Result<Vec<_>, _>>()?;
    let previous_by_key = previous_hub_projects
        .iter()
        .map(|project| (hub_recent_project_path_key(&project.path), project))
        .collect::<std::collections::BTreeMap<_, _>>();
    let current_keys = hub_projects
        .iter()
        .map(|project| hub_recent_project_path_key(&project.path))
        .collect::<std::collections::BTreeSet<_>>();

    for (path_key, project) in &previous_by_key {
        if !current_keys.contains(path_key) {
            registry.remove(&project.path);
        }
    }
    for project in hub_projects {
        let path_key = hub_recent_project_path_key(&project.path);
        let changed_by_hub = previous_by_key
            .get(&path_key)
            .is_none_or(|previous| recent_project_changed(previous, &project));
        if changed_by_hub {
            registry.record(project);
        }
    }

    registry
        .validate()
        .map_err(|source| SharedRecentProjectsError::Contract {
            path: registry_path.to_path_buf(),
            source,
        })?;
    write_registry(registry_path, &registry)?;
    Ok(registry
        .projects
        .into_iter()
        .map(recent_project_from_shared)
        .collect())
}

fn load_registry(registry_path: &Path) -> Result<HubRecentProjectsV1, SharedRecentProjectsError> {
    if !registry_path.exists() {
        return Ok(HubRecentProjectsV1::default());
    }
    let bytes = fs::read(registry_path).map_err(|source| SharedRecentProjectsError::Io {
        operation: "read shared recent-project registry",
        path: registry_path.to_path_buf(),
        source,
    })?;
    let registry = serde_json::from_slice::<HubRecentProjectsV1>(&bytes).map_err(|source| {
        SharedRecentProjectsError::Decode {
            path: registry_path.to_path_buf(),
            source,
        }
    })?;
    registry
        .validate()
        .map_err(|source| SharedRecentProjectsError::Contract {
            path: registry_path.to_path_buf(),
            source,
        })?;
    Ok(registry)
}

fn write_registry(
    registry_path: &Path,
    registry: &HubRecentProjectsV1,
) -> Result<(), SharedRecentProjectsError> {
    let parent = registry_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| SharedRecentProjectsError::Io {
        operation: "create shared recent-project registry directory",
        path: parent.to_path_buf(),
        source,
    })?;
    let bytes = serde_json::to_vec_pretty(registry).map_err(|source| {
        SharedRecentProjectsError::Encode {
            path: registry_path.to_path_buf(),
            source,
        }
    })?;
    write_atomic(registry_path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), SharedRecentProjectsError> {
    let temporary = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!("{extension}."))
            .unwrap_or_default()
    ));
    fs::write(&temporary, bytes).map_err(|source| SharedRecentProjectsError::Io {
        operation: "write shared recent-project registry staging file",
        path: temporary.clone(),
        source,
    })?;
    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(first_error) if path.exists() => {
            replace_existing_file(&temporary, path).map_err(|replace_error| {
                let _ = fs::remove_file(&temporary);
                SharedRecentProjectsError::AtomicReplace {
                    path: path.to_path_buf(),
                    first_error,
                    replace_error,
                }
            })
        }
        Err(source) => {
            let _ = fs::remove_file(&temporary);
            Err(SharedRecentProjectsError::Io {
                operation: "replace shared recent-project registry",
                path: path.to_path_buf(),
                source,
            })
        }
    }
}

fn shared_project_from_recent(
    registry_path: &Path,
    project: RecentProject,
) -> Result<HubRecentProjectV1, SharedRecentProjectsError> {
    HubRecentProjectV1::new(
        project.summary,
        normalize_project_root(project.path),
        project.last_opened_unix_ms,
    )
    .map_err(|source| SharedRecentProjectsError::Contract {
        path: registry_path.to_path_buf(),
        source,
    })
}

fn recent_project_from_shared(project: HubRecentProjectV1) -> RecentProject {
    RecentProject::from_summary(project.summary, project.path, project.last_opened_unix_ms)
}

fn recent_project_changed(previous: &RecentProject, current: &HubRecentProjectV1) -> bool {
    previous.summary != current.summary
        || previous.last_opened_unix_ms != current.last_opened_unix_ms
}

#[derive(Debug, Error)]
pub enum SharedRecentProjectsError {
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
    #[error(
        "failed to atomically replace shared recent-project registry `{path}`: rename: {first_error}; replacement: {replace_error}"
    )]
    AtomicReplace {
        path: PathBuf,
        first_error: io::Error,
        replace_error: io::Error,
    },
    #[error("shared recent-project writer lease is unsupported for `{path}`")]
    PlatformUnsupported { path: PathBuf },
}

#[derive(Debug)]
struct SharedRecentProjectsWriteLease {
    #[cfg(windows)]
    handle: isize,
    #[cfg(unix)]
    lock_file: fs::File,
}

impl SharedRecentProjectsWriteLease {
    fn acquire(registry_path: &Path) -> Result<Self, SharedRecentProjectsError> {
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
            Err(SharedRecentProjectsError::PlatformUnsupported {
                path: registry_path.to_path_buf(),
            })
        }
    }

    #[cfg(windows)]
    fn acquire_windows(registry_path: &Path) -> Result<Self, SharedRecentProjectsError> {
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
            return Err(SharedRecentProjectsError::Io {
                operation: "create shared recent-project writer lease",
                path: registry_path.to_path_buf(),
                source: io::Error::last_os_error(),
            });
        }
        // SAFETY: `handle` is valid and an abandoned mutex transfers ownership to this caller.
        match unsafe { WaitForSingleObject(handle, INFINITE) } {
            WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Self { handle }),
            _ => {
                // SAFETY: this local handle has not been transferred to a lease.
                unsafe {
                    CloseHandle(handle);
                }
                Err(SharedRecentProjectsError::Io {
                    operation: "acquire shared recent-project writer lease",
                    path: registry_path.to_path_buf(),
                    source: io::Error::last_os_error(),
                })
            }
        }
    }

    #[cfg(unix)]
    fn acquire_unix(registry_path: &Path) -> Result<Self, SharedRecentProjectsError> {
        use std::os::unix::io::AsRawFd;

        const LOCK_EX: i32 = 2;
        let lock_path = hub_recent_projects_lock_path(registry_path);
        let parent = lock_path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|source| SharedRecentProjectsError::Io {
            operation: "create shared recent-project registry directory",
            path: parent.to_path_buf(),
            source,
        })?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|source| SharedRecentProjectsError::Io {
                operation: "open shared recent-project writer lease",
                path: lock_path.clone(),
                source,
            })?;
        // SAFETY: the descriptor belongs to `lock_file`, retained by the returned lease.
        if unsafe { flock(lock_file.as_raw_fd(), LOCK_EX) } != 0 {
            return Err(SharedRecentProjectsError::Io {
                operation: "acquire shared recent-project writer lease",
                path: lock_path,
                source: io::Error::last_os_error(),
            });
        }
        Ok(Self { lock_file })
    }
}

#[cfg(windows)]
impl Drop for SharedRecentProjectsWriteLease {
    fn drop(&mut self) {
        // SAFETY: this lease owns the mutex after a successful wait.
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}

#[cfg(unix)]
impl Drop for SharedRecentProjectsWriteLease {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        const LOCK_UN: i32 = 8;
        // SAFETY: the descriptor remains valid until this destructor returns.
        unsafe {
            flock(self.lock_file.as_raw_fd(), LOCK_UN);
        }
    }
}

#[cfg(not(windows))]
fn replace_existing_file(temporary: &Path, target: &Path) -> io::Result<()> {
    fs::rename(temporary, target)
}

#[cfg(windows)]
fn replace_existing_file(temporary: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const REPLACEFILE_WRITE_THROUGH: u32 = 1;

    fn wide_path(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    let target = wide_path(target);
    let replacement = wide_path(temporary);
    // SAFETY: both paths are NUL-terminated for the duration of this synchronous call.
    if unsafe {
        ReplaceFileW(
            target.as_ptr(),
            replacement.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH,
            std::ptr::null(),
            std::ptr::null(),
        )
    } == 0
    {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
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
    fn ReplaceFileW(
        replaced_file_name: *const u16,
        replacement_file_name: *const u16,
        backup_file_name: *const u16,
        replace_flags: u32,
        exclude: *const std::ffi::c_void,
        reserved: *const std::ffi::c_void,
    ) -> i32;
}

#[cfg(unix)]
unsafe extern "C" {
    fn flock(file_descriptor: i32, operation: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::{load_shared_recent_projects, reconcile_shared_recent_projects};
    use crate::projects::RecentProject;

    #[test]
    fn reconciliation_merges_hub_history_with_the_shared_v1_registry() {
        let target_directory = std::env::var_os("CARGO_TARGET_DIR").expect(
            "shared recent-project filesystem tests require coordinator-managed CARGO_TARGET_DIR",
        );
        let root = std::path::PathBuf::from(target_directory).join(format!(
            "zircon-hub-shared-recents-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        let registry_path = root.join("recent_projects.json");

        let first = reconcile_shared_recent_projects(
            &registry_path,
            &[],
            &[RecentProject::fixture("Old", "E:/Projects/Game", 1)],
        )
        .expect("write initial shared history");
        let second = reconcile_shared_recent_projects(
            &registry_path,
            &first,
            &[
                RecentProject::fixture("Current", "e:/projects/game/", 9),
                RecentProject::fixture("Other", "E:/Projects/Other", 2),
            ],
        )
        .expect("merge shared history");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 2);
        assert_eq!(second[0].summary.name, "Current");
        assert_eq!(
            load_shared_recent_projects(&registry_path).expect("read shared history")[0]
                .summary
                .name,
            "Current"
        );
        std::fs::remove_dir_all(root).expect("remove shared history fixture");
    }

    #[test]
    fn stale_hub_snapshot_does_not_restore_an_editor_removed_project() {
        let target_directory = std::env::var_os("CARGO_TARGET_DIR").expect(
            "shared recent-project filesystem tests require coordinator-managed CARGO_TARGET_DIR",
        );
        let root = std::path::PathBuf::from(target_directory).join(format!(
            "zircon-hub-shared-recents-delete-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        let registry_path = root.join("recent_projects.json");
        let initial = reconcile_shared_recent_projects(
            &registry_path,
            &[],
            &[RecentProject::fixture("Game", "E:/Projects/Game", 1)],
        )
        .expect("write initial shared history");
        let removed = reconcile_shared_recent_projects(&registry_path, &initial, &[])
            .expect("record Editor-side removal");
        let after_stale_hub_persist =
            reconcile_shared_recent_projects(&registry_path, &initial, &initial)
                .expect("reconcile stale Hub snapshot");

        assert!(removed.is_empty());
        assert!(after_stale_hub_persist.is_empty());
        std::fs::remove_dir_all(root).expect("remove shared history fixture");
    }
}
