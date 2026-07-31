use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

use super::RuntimeLibraryError;

pub(crate) const ZIRCON_RUNTIME_LIBRARY_ENV: &str = "ZIRCON_RUNTIME_LIBRARY";

#[derive(Debug)]
pub(crate) enum RuntimeLibraryPathError {
    EnvironmentOverride(RuntimeLibraryError),
    DefaultResolution(RuntimeLibraryError),
}

#[derive(Debug)]
pub(crate) enum RuntimeLibraryPathSelection {
    EnvironmentOverride(PathBuf),
    Default(PathBuf),
}

impl RuntimeLibraryPathSelection {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::EnvironmentOverride(path) | Self::Default(path) => path,
        }
    }
}

impl Display for RuntimeLibraryPathError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentOverride(error) | Self::DefaultResolution(error) => {
                error.fmt(formatter)
            }
        }
    }
}

impl Error for RuntimeLibraryPathError {}

pub(crate) fn default_runtime_library_path(
) -> Result<RuntimeLibraryPathSelection, RuntimeLibraryPathError> {
    if let Some(path) =
        env_runtime_library_path().map_err(RuntimeLibraryPathError::EnvironmentOverride)?
    {
        return Ok(RuntimeLibraryPathSelection::EnvironmentOverride(path));
    }
    let executable = env::current_exe().map_err(|error| {
        RuntimeLibraryPathError::DefaultResolution(RuntimeLibraryError::new(format!(
            "failed to resolve current executable: {error}"
        )))
    })?;
    Ok(RuntimeLibraryPathSelection::Default(
        runtime_library_path_for_executable(&executable),
    ))
}

pub(crate) fn runtime_library_environment_override_request(path: &Path) -> String {
    format!("{ZIRCON_RUNTIME_LIBRARY_ENV}={}", path.display())
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

fn env_runtime_library_path() -> Result<Option<PathBuf>, RuntimeLibraryError> {
    runtime_library_override_path_from_value(env::var_os(ZIRCON_RUNTIME_LIBRARY_ENV))
}

fn runtime_library_override_path_from_value(
    value: Option<OsString>,
) -> Result<Option<PathBuf>, RuntimeLibraryError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    if value.to_str().is_some_and(|value| value.trim().is_empty()) {
        return Err(RuntimeLibraryError::new(format!(
            "runtime startup diagnostic: component=runtime_library requested_path=<environment override> cause={ZIRCON_RUNTIME_LIBRARY_ENV} is blank recovery=unset {ZIRCON_RUNTIME_LIBRARY_ENV} or set it to a compatible absolute path"
        )));
    }

    Ok(Some(PathBuf::from(value)))
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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::super::RuntimeLibraryError;
    use super::{
        runtime_library_environment_override_request, runtime_library_override_path_from_value,
        RuntimeLibraryPathError, RuntimeLibraryPathSelection,
    };

    #[test]
    fn runtime_library_override_path_rejects_blank_unicode_value() {
        let error = runtime_library_override_path_from_value(Some(OsString::from("\u{2003}")))
            .expect_err("blank runtime override must not be treated as a loadable path");

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_library requested_path=<environment override> cause=ZIRCON_RUNTIME_LIBRARY is blank recovery=unset ZIRCON_RUNTIME_LIBRARY or set it to a compatible absolute path"
        );
    }

    #[test]
    fn runtime_library_override_path_preserves_empty_fallback_and_valid_paths() {
        assert_eq!(
            runtime_library_override_path_from_value(Some(OsString::new())).unwrap(),
            None
        );
        assert_eq!(
            runtime_library_override_path_from_value(Some(OsString::from("runtime.dll"))).unwrap(),
            Some(PathBuf::from("runtime.dll"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn runtime_library_override_path_preserves_non_utf8_value() {
        use std::os::unix::ffi::OsStringExt;

        let value = OsString::from_vec(vec![0xFF]);

        assert_eq!(
            runtime_library_override_path_from_value(Some(value.clone())).unwrap(),
            Some(PathBuf::from(value))
        );
    }

    #[test]
    fn runtime_library_path_error_preserves_override_diagnostic() {
        let error = RuntimeLibraryPathError::EnvironmentOverride(RuntimeLibraryError::new(
            "environment override diagnostic",
        ));

        assert_eq!(error.to_string(), "environment override diagnostic");
    }

    #[test]
    fn runtime_library_override_selection_keeps_its_provenance_and_request_label() {
        let selection =
            RuntimeLibraryPathSelection::EnvironmentOverride(PathBuf::from("custom.dll"));

        assert_eq!(selection.path(), std::path::Path::new("custom.dll"));
        assert_eq!(
            runtime_library_environment_override_request(selection.path()),
            "ZIRCON_RUNTIME_LIBRARY=custom.dll"
        );
    }
}
