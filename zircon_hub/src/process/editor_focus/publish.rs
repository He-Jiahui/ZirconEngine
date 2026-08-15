use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime_interface::hub_protocol::{
    hub_editor_focus_signal_path, HubEditorFocusSignalV1, HubSessionToken,
};
use zircon_runtime_interface::project::session_lock::ProjectSessionLockRecordV1;

use crate::error::HubError;

static NEXT_FOCUS_WRITE: AtomicU64 = AtomicU64::new(1);

/// Atomically publishes a focus request without touching the editor's recovery lock.
pub(crate) fn publish_project_editor_focus_signal(
    project_root: impl AsRef<Path>,
    target: &ProjectSessionLockRecordV1,
    session: HubSessionToken,
) -> Result<(), HubError> {
    let mailbox_path = hub_editor_focus_signal_path(project_root, target.instance_id())
        .map_err(|source| HubError::message(source.to_string()))?;
    let bytes = serde_json::to_vec(&HubEditorFocusSignalV1::new(session, target.instance_id()))?;
    write_atomically(&mailbox_path, &bytes).map_err(|source| {
        HubError::message(format!(
            "publish editor focus signal `{}`: {source}",
            mailbox_path.display()
        ))
    })
}

fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let directory = path
        .parent()
        .expect("focus mailbox always has a parent directory");
    fs::create_dir_all(directory)?;

    for _ in 0..2 {
        let staging = focus_staging_path(path);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)?;
        if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            let _ = fs::remove_file(&staging);
            return Err(error);
        }
        drop(file);

        match publish_staging_file(&staging, path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound && !path.exists() => {
                let _ = fs::remove_file(&staging);
            }
            Err(error) => {
                let _ = fs::remove_file(&staging);
                return Err(error);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "focus mailbox changed while replacing its atomic payload",
    ))
}

fn focus_staging_path(path: &Path) -> PathBuf {
    let sequence = NEXT_FOCUS_WRITE.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("focus.json");
    path.with_file_name(format!(
        ".{file_name}.zr-focus-{}-{sequence}.tmp",
        std::process::id()
    ))
}

#[cfg(not(windows))]
fn publish_staging_file(staging: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(staging, destination)
}

#[cfg(windows)]
fn publish_staging_file(staging: &Path, destination: &Path) -> io::Result<()> {
    match fs::rename(staging, destination) {
        Ok(()) => Ok(()),
        Err(error) if destination.exists() => replace_existing_file(staging, destination),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
fn replace_existing_file(staging: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

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
    // SAFETY: both filenames are NUL-terminated and the operation uses no backup path.
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

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn ReplaceFileW(
        destination: *const u16,
        replacement: *const u16,
        backup: *const u16,
        replace_flags: u32,
        exclude: *mut std::ffi::c_void,
        reserved: *mut std::ffi::c_void,
    ) -> i32;
}
