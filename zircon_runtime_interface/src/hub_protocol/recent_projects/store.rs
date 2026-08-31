use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use thiserror::Error;
use uuid::Uuid;

#[cfg(windows)]
use super::windows_hub_recent_projects_mutex_name;
use super::{hub_recent_projects_lock_path, HubRecentProjectsError, HubRecentProjectsV1};

/// The shared history has at most eight entries and is only a rebuildable UI projection.
pub const HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1: usize = 256 * 1024;
const LEASE_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// A bounded write attempt for the rebuildable shared recent-project projection.
pub struct HubRecentProjectsWritePolicy<'a> {
    deadline: Instant,
    cancellation: Option<&'a dyn Fn() -> bool>,
    nonblocking: bool,
}

impl<'a> HubRecentProjectsWritePolicy<'a> {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            deadline: Instant::now() + timeout,
            cancellation: None,
            nonblocking: false,
        }
    }

    pub fn try_now() -> Self {
        Self {
            deadline: Instant::now(),
            cancellation: None,
            nonblocking: true,
        }
    }

    pub fn with_cancellation(mut self, cancellation: &'a dyn Fn() -> bool) -> Self {
        self.cancellation = Some(cancellation);
        self
    }

    fn cancelled(&self) -> bool {
        self.cancellation.is_some_and(|cancellation| cancellation())
    }
}

/// How a read-only projection was obtained without making project open depend on history health.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HubRecentProjectsLoadDisposition {
    Clean,
    RebuildRequiredAfterCorruption,
    RebuildRequiredAfterOversize,
}

/// A read result that preserves the registry health outcome for host diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubRecentProjectsLoad {
    registry: HubRecentProjectsV1,
    disposition: HubRecentProjectsLoadDisposition,
}

impl HubRecentProjectsLoad {
    pub fn registry(&self) -> &HubRecentProjectsV1 {
        &self.registry
    }

    pub const fn disposition(&self) -> HubRecentProjectsLoadDisposition {
        self.disposition
    }
}

/// A completed write, including whether it isolated malformed input before rebuilding it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubRecentProjectsMutation {
    registry: HubRecentProjectsV1,
    previous_revision: u64,
    load_disposition: HubRecentProjectsLoadDisposition,
    quarantined_path: Option<PathBuf>,
}

impl HubRecentProjectsMutation {
    pub fn registry(&self) -> &HubRecentProjectsV1 {
        &self.registry
    }

    /// The revision read under the writer lease before this mutation was applied.
    pub const fn previous_revision(&self) -> u64 {
        self.previous_revision
    }

    pub const fn load_disposition(&self) -> HubRecentProjectsLoadDisposition {
        self.load_disposition
    }

    pub fn quarantined_path(&self) -> Option<&Path> {
        self.quarantined_path.as_deref()
    }
}

/// The one shared file transaction owner used by Hub and Editor recent-project projections.
#[derive(Clone, Debug)]
pub struct HubRecentProjectsStore {
    registry_path: PathBuf,
}

impl HubRecentProjectsStore {
    pub fn new(registry_path: impl Into<PathBuf>) -> Self {
        Self {
            registry_path: registry_path.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.registry_path
    }

    /// Reads only a bounded projection. Malformed and oversized history is intentionally treated
    /// as an empty rebuildable projection so it cannot block canonical project open.
    pub fn load_projection(&self) -> Result<HubRecentProjectsLoad, HubRecentProjectsStoreError> {
        match read_registry(&self.registry_path) {
            Ok(registry) => Ok(HubRecentProjectsLoad {
                registry,
                disposition: HubRecentProjectsLoadDisposition::Clean,
            }),
            Err(error) if error.requires_rebuild() => Ok(HubRecentProjectsLoad {
                registry: HubRecentProjectsV1::default(),
                disposition: error.rebuild_disposition(),
            }),
            Err(error) => Err(error),
        }
    }

    /// Applies one bounded read-merge-write transaction. Corrupt or oversized input is first
    /// quarantined under the same lease, then rebuilt from the caller's new projection.
    pub fn update(
        &self,
        policy: HubRecentProjectsWritePolicy<'_>,
        update: impl FnOnce(&mut HubRecentProjectsV1) -> Result<(), HubRecentProjectsError>,
    ) -> Result<HubRecentProjectsMutation, HubRecentProjectsStoreError> {
        self.update_at_revision(policy, None, update)
    }

    /// Applies an update only when the projection still has the revision the caller inspected.
    ///
    /// The caller must re-read and rebase after `RevisionConflict`; this prevents a stale
    /// in-memory Hub snapshot from silently overwriting a newer Editor-side projection.
    pub fn compare_and_update(
        &self,
        policy: HubRecentProjectsWritePolicy<'_>,
        expected_revision: u64,
        update: impl FnOnce(&mut HubRecentProjectsV1) -> Result<(), HubRecentProjectsError>,
    ) -> Result<HubRecentProjectsMutation, HubRecentProjectsStoreError> {
        self.update_at_revision(policy, Some(expected_revision), update)
    }

    fn update_at_revision(
        &self,
        policy: HubRecentProjectsWritePolicy<'_>,
        expected_revision: Option<u64>,
        update: impl FnOnce(&mut HubRecentProjectsV1) -> Result<(), HubRecentProjectsError>,
    ) -> Result<HubRecentProjectsMutation, HubRecentProjectsStoreError> {
        let _lease = HubRecentProjectsWriteLease::acquire(&self.registry_path, &policy)?;
        let load = self.load_projection()?;
        let previous_revision = load.registry().revision();
        if let Some(expected_revision) = expected_revision {
            if expected_revision != previous_revision {
                return Err(HubRecentProjectsStoreError::RevisionConflict {
                    path: self.registry_path.clone(),
                    expected_revision,
                    actual_revision: previous_revision,
                });
            }
        }
        let quarantined_path = match load.disposition() {
            HubRecentProjectsLoadDisposition::Clean => None,
            HubRecentProjectsLoadDisposition::RebuildRequiredAfterCorruption
            | HubRecentProjectsLoadDisposition::RebuildRequiredAfterOversize => {
                Some(quarantine_registry(&self.registry_path)?)
            }
        };
        let mut registry = load.registry().clone();
        update(&mut registry).map_err(|source| HubRecentProjectsStoreError::Contract {
            path: self.registry_path.clone(),
            source,
        })?;
        registry
            .validate()
            .map_err(|source| HubRecentProjectsStoreError::Contract {
                path: self.registry_path.clone(),
                source,
            })?;
        write_registry(&self.registry_path, &registry)?;
        Ok(HubRecentProjectsMutation {
            registry,
            previous_revision,
            load_disposition: load.disposition(),
            quarantined_path,
        })
    }
}

fn read_registry(path: &Path) -> Result<HubRecentProjectsV1, HubRecentProjectsStoreError> {
    let Some(bytes) = read_bounded(path)? else {
        return Ok(HubRecentProjectsV1::default());
    };
    let registry = serde_json::from_slice::<HubRecentProjectsV1>(&bytes).map_err(|source| {
        HubRecentProjectsStoreError::Decode {
            path: path.to_path_buf(),
            source,
        }
    })?;
    registry
        .validate()
        .map_err(|source| HubRecentProjectsStoreError::Contract {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(registry)
}

fn read_bounded(path: &Path) -> Result<Option<Vec<u8>>, HubRecentProjectsStoreError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(source) => {
            return Err(HubRecentProjectsStoreError::Io {
                operation: "open shared recent-project registry",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let declared_bytes = file
        .metadata()
        .map_err(|source| HubRecentProjectsStoreError::Io {
            operation: "inspect shared recent-project registry",
            path: path.to_path_buf(),
            source,
        })?
        .len();
    if declared_bytes > HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1 as u64 {
        return Err(HubRecentProjectsStoreError::Oversized {
            path: path.to_path_buf(),
            actual_bytes: declared_bytes,
            max_bytes: HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1,
        });
    }
    let mut bytes = Vec::with_capacity(declared_bytes as usize);
    Read::by_ref(&mut file)
        .take(HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1 as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| HubRecentProjectsStoreError::Io {
            operation: "read shared recent-project registry",
            path: path.to_path_buf(),
            source,
        })?;
    if bytes.len() > HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1 {
        return Err(HubRecentProjectsStoreError::Oversized {
            path: path.to_path_buf(),
            actual_bytes: bytes.len() as u64,
            max_bytes: HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1,
        });
    }
    Ok(Some(bytes))
}

fn write_registry(
    path: &Path,
    registry: &HubRecentProjectsV1,
) -> Result<(), HubRecentProjectsStoreError> {
    let bytes = serde_json::to_vec_pretty(registry).map_err(|source| {
        HubRecentProjectsStoreError::Encode {
            path: path.to_path_buf(),
            source,
        }
    })?;
    if bytes.len() > HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1 {
        return Err(HubRecentProjectsStoreError::Oversized {
            path: path.to_path_buf(),
            actual_bytes: bytes.len() as u64,
            max_bytes: HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1,
        });
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| HubRecentProjectsStoreError::Io {
        operation: "create shared recent-project registry directory",
        path: parent.to_path_buf(),
        source,
    })?;
    let temporary = parent.join(format!(
        ".{}-{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("recent"),
        Uuid::new_v4(),
    ));
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        replace_file(&temporary, path)?;
        sync_parent_directory(parent)?;
        Ok::<(), io::Error>(())
    })();
    if let Err(source) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(HubRecentProjectsStoreError::Io {
            operation: "atomically publish shared recent-project registry",
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn quarantine_registry(path: &Path) -> Result<PathBuf, HubRecentProjectsStoreError> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("recent_projects.json");
    let quarantine = parent.join(format!("{file_name}.corrupt-{}.json", Uuid::new_v4()));
    fs::rename(path, &quarantine).map_err(|source| HubRecentProjectsStoreError::Io {
        operation: "quarantine malformed shared recent-project registry",
        path: path.to_path_buf(),
        source,
    })?;
    sync_parent_directory(parent).map_err(|source| HubRecentProjectsStoreError::Io {
        operation: "sync shared recent-project registry quarantine directory",
        path: parent.to_path_buf(),
        source,
    })?;
    Ok(quarantine)
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, target: &Path) -> io::Result<()> {
    fs::rename(temporary, target)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const REPLACEFILE_WRITE_THROUGH: u32 = 1;

    fn wide_path(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    if !target.exists() {
        return fs::rename(temporary, target);
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

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> io::Result<()> {
    Ok(())
}

#[derive(Debug)]
struct HubRecentProjectsWriteLease {
    #[cfg(windows)]
    handle: isize,
    #[cfg(unix)]
    lock_file: File,
}

impl HubRecentProjectsWriteLease {
    fn acquire(
        registry_path: &Path,
        policy: &HubRecentProjectsWritePolicy<'_>,
    ) -> Result<Self, HubRecentProjectsStoreError> {
        #[cfg(windows)]
        {
            return Self::acquire_windows(registry_path, policy);
        }
        #[cfg(unix)]
        {
            return Self::acquire_unix(registry_path, policy);
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = policy;
            Err(HubRecentProjectsStoreError::PlatformUnsupported {
                path: registry_path.to_path_buf(),
            })
        }
    }

    #[cfg(windows)]
    fn acquire_windows(
        registry_path: &Path,
        policy: &HubRecentProjectsWritePolicy<'_>,
    ) -> Result<Self, HubRecentProjectsStoreError> {
        const WAIT_OBJECT_0: u32 = 0;
        const WAIT_ABANDONED: u32 = 0x0000_0080;
        const WAIT_TIMEOUT: u32 = 0x0000_0102;

        let started = Instant::now();
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
        loop {
            if policy.cancelled() {
                unsafe {
                    CloseHandle(handle);
                }
                return Err(HubRecentProjectsStoreError::LeaseCancelled {
                    path: registry_path.to_path_buf(),
                });
            }
            if policy.nonblocking {
                // SAFETY: `handle` is valid and a zero timeout never blocks the caller.
                match unsafe { WaitForSingleObject(handle, 0) } {
                    WAIT_OBJECT_0 | WAIT_ABANDONED => return Ok(Self { handle }),
                    WAIT_TIMEOUT => {
                        unsafe {
                            CloseHandle(handle);
                        }
                        return Err(HubRecentProjectsStoreError::LeaseDeadlineExceeded {
                            path: registry_path.to_path_buf(),
                            waited: started.elapsed(),
                        });
                    }
                    _ => {
                        unsafe {
                            CloseHandle(handle);
                        }
                        return Err(HubRecentProjectsStoreError::Io {
                            operation: "acquire shared recent-project writer lease",
                            path: registry_path.to_path_buf(),
                            source: io::Error::last_os_error(),
                        });
                    }
                }
            }
            let remaining = deadline_remaining(registry_path, started, policy)?;
            let wait_millis = remaining.min(LEASE_POLL_INTERVAL).as_millis().max(1) as u32;
            // SAFETY: `handle` is valid and an abandoned mutex transfers ownership to this caller.
            match unsafe { WaitForSingleObject(handle, wait_millis) } {
                WAIT_OBJECT_0 | WAIT_ABANDONED => return Ok(Self { handle }),
                WAIT_TIMEOUT => continue,
                _ => {
                    unsafe {
                        CloseHandle(handle);
                    }
                    return Err(HubRecentProjectsStoreError::Io {
                        operation: "acquire shared recent-project writer lease",
                        path: registry_path.to_path_buf(),
                        source: io::Error::last_os_error(),
                    });
                }
            }
        }
    }

    #[cfg(unix)]
    fn acquire_unix(
        registry_path: &Path,
        policy: &HubRecentProjectsWritePolicy<'_>,
    ) -> Result<Self, HubRecentProjectsStoreError> {
        use std::os::unix::io::AsRawFd;

        const LOCK_EX: i32 = 2;
        const LOCK_NB: i32 = 4;

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
        let started = Instant::now();
        loop {
            if policy.cancelled() {
                return Err(HubRecentProjectsStoreError::LeaseCancelled {
                    path: registry_path.to_path_buf(),
                });
            }
            // SAFETY: the descriptor belongs to `lock_file`, retained by the returned lease.
            if unsafe { flock(lock_file.as_raw_fd(), LOCK_EX | LOCK_NB) } == 0 {
                return Ok(Self { lock_file });
            }
            let source = io::Error::last_os_error();
            if source.kind() != io::ErrorKind::WouldBlock {
                return Err(HubRecentProjectsStoreError::Io {
                    operation: "acquire shared recent-project writer lease",
                    path: lock_path,
                    source,
                });
            }
            if policy.nonblocking {
                return Err(HubRecentProjectsStoreError::LeaseDeadlineExceeded {
                    path: registry_path.to_path_buf(),
                    waited: started.elapsed(),
                });
            }
            let remaining = deadline_remaining(registry_path, started, policy)?;
            thread::sleep(remaining.min(LEASE_POLL_INTERVAL));
        }
    }
}

fn deadline_remaining(
    path: &Path,
    started: Instant,
    policy: &HubRecentProjectsWritePolicy<'_>,
) -> Result<Duration, HubRecentProjectsStoreError> {
    let now = Instant::now();
    if now >= policy.deadline {
        return Err(HubRecentProjectsStoreError::LeaseDeadlineExceeded {
            path: path.to_path_buf(),
            waited: now.saturating_duration_since(started),
        });
    }
    Ok(policy.deadline.saturating_duration_since(now))
}

#[cfg(windows)]
impl Drop for HubRecentProjectsWriteLease {
    fn drop(&mut self) {
        // SAFETY: this lease owns the mutex after a successful wait.
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}

#[cfg(unix)]
impl Drop for HubRecentProjectsWriteLease {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        const LOCK_UN: i32 = 8;
        // SAFETY: the descriptor remains valid until this destructor returns.
        unsafe {
            flock(self.lock_file.as_raw_fd(), LOCK_UN);
        }
    }
}

#[derive(Debug, Error)]
pub enum HubRecentProjectsStoreError {
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
    #[error("shared recent-project registry `{path}` violates the v1 contract: {source}")]
    Contract {
        path: PathBuf,
        #[source]
        source: HubRecentProjectsError,
    },
    #[error(
        "shared recent-project registry `{path}` is {actual_bytes} bytes, above the {max_bytes}-byte limit"
    )]
    Oversized {
        path: PathBuf,
        actual_bytes: u64,
        max_bytes: usize,
    },
    #[error("failed to encode shared recent-project registry `{path}`: {source}")]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("shared recent-project writer lease expired for `{path}` after {waited:?}")]
    LeaseDeadlineExceeded { path: PathBuf, waited: Duration },
    #[error("shared recent-project writer lease was cancelled for `{path}`")]
    LeaseCancelled { path: PathBuf },
    #[error(
        "shared recent-project registry `{path}` changed from expected revision {expected_revision} to {actual_revision}"
    )]
    RevisionConflict {
        path: PathBuf,
        expected_revision: u64,
        actual_revision: u64,
    },
    #[error("shared recent-project writer lease is unsupported for `{path}`")]
    PlatformUnsupported { path: PathBuf },
}

impl HubRecentProjectsStoreError {
    fn requires_rebuild(&self) -> bool {
        matches!(
            self,
            Self::Decode { .. } | Self::Contract { .. } | Self::Oversized { .. }
        )
    }

    fn rebuild_disposition(&self) -> HubRecentProjectsLoadDisposition {
        match self {
            Self::Oversized { .. } => {
                HubRecentProjectsLoadDisposition::RebuildRequiredAfterOversize
            }
            Self::Decode { .. } | Self::Contract { .. } => {
                HubRecentProjectsLoadDisposition::RebuildRequiredAfterCorruption
            }
            Self::Io { .. }
            | Self::Encode { .. }
            | Self::LeaseDeadlineExceeded { .. }
            | Self::LeaseCancelled { .. }
            | Self::RevisionConflict { .. }
            | Self::PlatformUnsupported { .. } => {
                unreachable!("only rebuildable registry errors reach this method")
            }
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
    use std::fs;
    use std::time::SystemTime;

    use crate::project::{ProjectManifestSummary, PROJECT_MANIFEST_FORMAT_VERSION};

    use super::{
        HubRecentProjectsLoadDisposition, HubRecentProjectsStore, HubRecentProjectsStoreError,
        HubRecentProjectsWritePolicy, HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1,
    };
    use crate::hub_protocol::HubRecentProjectV1;

    #[test]
    fn corrupted_projection_is_quarantined_and_rebuilt_during_a_bounded_mutation() {
        let root = temporary_root("corrupt");
        let path = root.join("recent_projects.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&path, b"not-json").expect("write corrupt fixture");
        let store = HubRecentProjectsStore::new(path.clone());

        let result = store
            .update(
                HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_millis(50)),
                |registry| registry.record(project()),
            )
            .expect("rebuild corrupt recent projection");

        assert_eq!(
            result.load_disposition(),
            HubRecentProjectsLoadDisposition::RebuildRequiredAfterCorruption
        );
        assert!(result.quarantined_path().is_some());
        assert_eq!(result.registry().projects, vec![project()]);
        assert_eq!(store.load_projection().unwrap().registry().revision(), 1);
        fs::remove_dir_all(root).expect("remove fixture root");
    }

    #[test]
    fn oversized_projection_is_nonblocking_and_reports_a_rebuild_requirement() {
        let root = temporary_root("oversized");
        let path = root.join("recent_projects.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(
            &path,
            vec![b'x'; HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1 + 1],
        )
        .expect("write oversized fixture");
        let store = HubRecentProjectsStore::new(path);

        let projection = store.load_projection().expect("bounded projection result");

        assert_eq!(
            projection.disposition(),
            HubRecentProjectsLoadDisposition::RebuildRequiredAfterOversize
        );
        assert!(projection.registry().projects.is_empty());
        fs::remove_dir_all(root).expect("remove fixture root");
    }

    #[test]
    fn cancelled_write_returns_a_typed_terminal_error_before_mutation() {
        let root = temporary_root("cancelled");
        let store = HubRecentProjectsStore::new(root.join("recent_projects.json"));
        let cancelled = || true;

        let error = store
            .update(
                HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_secs(1))
                    .with_cancellation(&cancelled),
                |registry| registry.record(project()),
            )
            .expect_err("cancelled writer must not mutate history");

        assert!(matches!(
            error,
            HubRecentProjectsStoreError::LeaseCancelled { .. }
        ));
        assert!(!store.path().exists());
    }

    #[test]
    fn compare_and_update_rejects_a_stale_projection_revision() {
        let root = temporary_root("stale-revision");
        let store = HubRecentProjectsStore::new(root.join("recent_projects.json"));
        let first = store
            .update(
                HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_millis(50)),
                |registry| registry.record(project()),
            )
            .expect("write initial projection");

        let error = store
            .compare_and_update(
                HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_millis(50)),
                first.previous_revision(),
                |registry| registry.remove("E:/Projects/Game"),
            )
            .expect_err("stale revision must not replace a newer projection");

        assert!(matches!(
            error,
            HubRecentProjectsStoreError::RevisionConflict {
                expected_revision: 0,
                actual_revision: 1,
                ..
            }
        ));
        std::fs::remove_dir_all(root).expect("remove fixture root");
    }

    fn project() -> HubRecentProjectV1 {
        HubRecentProjectV1::new(
            ProjectManifestSummary {
                name: "Game".to_string(),
                engine_version_req: None,
                default_scene: "res://scenes/main.scene.toml".to_string(),
                format_version: PROJECT_MANIFEST_FORMAT_VERSION,
            },
            "E:/Projects/Game",
            42,
        )
        .expect("fixture recent project")
    }

    fn temporary_root(label: &str) -> PathBuf {
        let target_directory = std::env::var_os("CARGO_TARGET_DIR")
            .expect("recent-project filesystem tests require coordinator-managed CARGO_TARGET_DIR");
        PathBuf::from(target_directory).join(format!(
            "zircon-recent-store-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("current time")
                .as_nanos(),
        ))
    }

    use std::path::PathBuf;
}
