use std::path::Path;

#[cfg(unix)]
use std::fs::File;

use crate::projects::normalize_project_root;

/// Holds the same exclusive project lease that Editor uses so a sync cannot race Editor startup.
/// The lease is process-local and is retained by the caller for the full project mutation.
pub(crate) struct ProjectCloudSyncLease {
    #[cfg(windows)]
    handle: isize,
    #[cfg(unix)]
    directory: File,
}

impl ProjectCloudSyncLease {
    pub(crate) fn acquire(project_root: &Path) -> Result<Self, &'static str> {
        let root = project_root
            .canonicalize()
            .map_err(|_| "hub_cloud_sync_project_busy")?;
        let root = normalize_project_root(root);

        #[cfg(windows)]
        {
            Self::acquire_windows(&root)
        }
        #[cfg(unix)]
        {
            Self::acquire_unix(&root)
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = root;
            Err("hub_cloud_sync_project_busy")
        }
    }

    #[cfg(windows)]
    fn acquire_windows(project_root: &Path) -> Result<Self, &'static str> {
        use std::io;

        const ERROR_ALREADY_EXISTS: i32 = 183;
        let name =
            zircon_runtime_interface::project::session_lock::windows_project_session_mutex_name(
                project_root,
            );
        let name = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        // SAFETY: the name is NUL-terminated and CreateMutexW returns an owned handle.
        let handle = unsafe {
            SetLastError(0);
            CreateMutexW(std::ptr::null(), 0, name.as_ptr())
        };
        if handle == 0 {
            return Err("hub_cloud_sync_project_busy");
        }
        if io::Error::last_os_error().raw_os_error() == Some(ERROR_ALREADY_EXISTS) {
            // SAFETY: CreateMutexW returned a valid handle to the pre-existing lease object.
            unsafe {
                CloseHandle(handle);
            }
            return Err("hub_cloud_sync_project_busy");
        }
        Ok(Self { handle })
    }

    #[cfg(unix)]
    fn acquire_unix(project_root: &Path) -> Result<Self, &'static str> {
        use std::os::unix::io::AsRawFd;

        const LOCK_EX: i32 = 2;
        const LOCK_NB: i32 = 4;
        let directory =
            File::open(project_root.join(".zircon")).map_err(|_| "hub_cloud_sync_project_busy")?;
        // SAFETY: the directory descriptor is retained in the returned lease through drop.
        if unsafe { flock(directory.as_raw_fd(), LOCK_EX | LOCK_NB) } != 0 {
            return Err("hub_cloud_sync_project_busy");
        }
        Ok(Self { directory })
    }
}

#[cfg(windows)]
impl Drop for ProjectCloudSyncLease {
    fn drop(&mut self) {
        // SAFETY: this handle belongs to the lease and is closed once.
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[cfg(unix)]
impl Drop for ProjectCloudSyncLease {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;

        const LOCK_UN: i32 = 8;
        // SAFETY: the directory descriptor remains valid until this destructor returns.
        unsafe {
            flock(self.directory.as_raw_fd(), LOCK_UN);
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
    fn CloseHandle(handle: isize) -> i32;
    fn SetLastError(error: u32);
}

#[cfg(unix)]
unsafe extern "C" {
    fn flock(file_descriptor: i32, operation: i32) -> i32;
}

#[cfg(test)]
#[path = "tests/sync.rs"]
mod tests;
