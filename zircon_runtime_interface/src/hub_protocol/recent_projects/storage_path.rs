use std::path::{Path, PathBuf};

const HUB_RECENT_PROJECTS_FILE_NAME: &str = "recent_projects.json";

/// Returns the shared Hub and Editor registry path required by protocol v1.
pub fn hub_recent_projects_path() -> PathBuf {
    let home = if cfg!(target_os = "windows") {
        std::env::var_os("USERPROFILE")
    } else {
        std::env::var_os("HOME")
    };
    home.map(|home| hub_recent_projects_path_from_home(PathBuf::from(home)))
        .unwrap_or_else(|| {
            PathBuf::from(".zircon")
                .join("hub")
                .join(HUB_RECENT_PROJECTS_FILE_NAME)
        })
}

/// Builds the protocol v1 path from an explicit home directory for deterministic host tests.
pub fn hub_recent_projects_path_from_home(home: impl AsRef<Path>) -> PathBuf {
    home.as_ref()
        .join(".zircon")
        .join("hub")
        .join(HUB_RECENT_PROJECTS_FILE_NAME)
}

/// Sidecar file used by non-Windows hosts for a short exclusive read-merge-write lease.
pub fn hub_recent_projects_lock_path(recent_projects_path: impl AsRef<Path>) -> PathBuf {
    let recent_projects_path = recent_projects_path.as_ref();
    let file_name = recent_projects_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(HUB_RECENT_PROJECTS_FILE_NAME);
    recent_projects_path.with_file_name(format!(".{file_name}.lock"))
}
