use std::path::Path;

/// Deterministic, lexical project identity used only for the shared recent-project registry.
///
/// Hosts must supply canonical display paths when a project exists. This function intentionally
/// does not perform filesystem I/O so the DTO layer remains usable in all runtime hosts.
pub fn hub_recent_project_path_key(path: impl AsRef<Path>) -> String {
    let mut text = path.as_ref().to_string_lossy().replace('\\', "/");
    while text.ends_with('/') && text.len() > 1 {
        text.pop();
    }
    if cfg!(target_os = "windows") || looks_like_windows_drive_path(&text) {
        text.make_ascii_lowercase();
    }
    text
}

#[cfg(windows)]
pub fn windows_hub_recent_projects_mutex_name(recent_projects_path: impl AsRef<Path>) -> String {
    use std::os::windows::ffi::OsStrExt;

    let mut bytes = Vec::new();
    for unit in recent_projects_path.as_ref().as_os_str().encode_wide() {
        bytes.extend(unit.to_le_bytes());
    }
    format!(
        "Global\\ZirconEngineHubRecentProjects-{}",
        blake3::hash(&bytes).to_hex()
    )
}

fn looks_like_windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
