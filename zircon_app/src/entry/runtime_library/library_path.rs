use std::env;
use std::path::{Path, PathBuf};

use super::RuntimeLibraryError;

pub(crate) const ZIRCON_RUNTIME_LIBRARY_ENV: &str = "ZIRCON_RUNTIME_LIBRARY";

pub(crate) fn default_runtime_library_path() -> Result<PathBuf, RuntimeLibraryError> {
    if let Some(path) = env_runtime_library_path() {
        return Ok(path);
    }
    let executable = env::current_exe().map_err(|error| {
        RuntimeLibraryError::new(format!("failed to resolve current executable: {error}"))
    })?;
    Ok(runtime_library_path_for_executable(&executable))
}

pub(super) fn runtime_library_path_for_executable(executable: &Path) -> PathBuf {
    let sibling = executable.with_file_name(platform_runtime_library_name());
    if sibling.exists() {
        return sibling;
    }

    executable
        .parent()
        .map(|parent| parent.join("deps").join(platform_runtime_library_name()))
        .filter(|candidate| candidate.exists())
        .unwrap_or(sibling)
}

fn env_runtime_library_path() -> Option<PathBuf> {
    env::var_os(ZIRCON_RUNTIME_LIBRARY_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub(crate) const fn platform_runtime_library_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "zircon_runtime.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "libzircon_runtime.dylib"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "libzircon_runtime.so"
    }
}
