use std::path::{Path, PathBuf};

pub const PROJECT_SESSION_LOCK_FILE_NAME: &str = "session.lock";

pub fn project_session_lock_path(project_root: impl AsRef<Path>) -> PathBuf {
    project_root
        .as_ref()
        .join(".zircon")
        .join(PROJECT_SESSION_LOCK_FILE_NAME)
}

#[cfg(windows)]
pub fn windows_project_session_mutex_name(project_root: impl AsRef<Path>) -> String {
    use std::os::windows::ffi::OsStrExt;

    let mut bytes = Vec::new();
    for unit in project_root.as_ref().as_os_str().encode_wide() {
        bytes.extend(unit.to_le_bytes());
    }
    format!(
        "Global\\ZirconEngineProjectSession-{}",
        blake3::hash(&bytes).to_hex()
    )
}
