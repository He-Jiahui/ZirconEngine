use std::path::PathBuf;

pub(crate) fn default_project_location() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(home) = std::env::var_os("USERPROFILE") {
            return PathBuf::from(home).join("Documents").join("ZirconProjects");
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("ZirconProjects");
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
