use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::ProjectPaths;

use super::RuntimeLibraryError;

pub(crate) const ZIRCON_RUNTIME_LIBRARY_ENV: &str = "ZIRCON_RUNTIME_LIBRARY";

#[derive(Debug)]
pub(crate) enum RuntimeLibraryPathError {
    EnvironmentOverride(RuntimeLibraryError),
    DefaultResolution(RuntimeLibraryError),
}

#[derive(Debug)]
pub(crate) enum RuntimeLibraryPathSelection {
    EnvironmentOverride { path: PathBuf, request: String },
    Default(PathBuf),
}

impl RuntimeLibraryPathSelection {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::EnvironmentOverride { path, .. } | Self::Default(path) => path,
        }
    }

    #[cfg(test)]
    fn environment_override_request(&self) -> Option<&str> {
        match self {
            Self::EnvironmentOverride { request, .. } => Some(request),
            Self::Default(_) => None,
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
    if let Some(selection) =
        env_runtime_library_path().map_err(RuntimeLibraryPathError::EnvironmentOverride)?
    {
        return Ok(selection);
    }
    let executable = env::current_exe().map_err(|error| {
        RuntimeLibraryPathError::DefaultResolution(RuntimeLibraryError::new(format!(
            "failed to resolve current executable: {error}"
        )))
    })?;
    runtime_library_path_for_executable(&executable)
        .map(RuntimeLibraryPathSelection::Default)
        .map_err(RuntimeLibraryPathError::DefaultResolution)
}

pub(crate) fn runtime_library_environment_override_request(path: &Path) -> String {
    format!("{ZIRCON_RUNTIME_LIBRARY_ENV}={}", path.display())
}

pub(super) fn runtime_library_path_for_executable(
    executable: &Path,
) -> Result<PathBuf, RuntimeLibraryError> {
    let directory = executable.parent().ok_or_else(|| {
        RuntimeLibraryError::new(
            "product executable has no parent directory for default runtime library",
        )
    })?;
    let product_directory = ProjectPaths::resolve_path(directory).map_err(|error| {
        RuntimeLibraryError::new(format!(
            "could not resolve product executable directory for default runtime library: {error}"
        ))
    })?;
    let sibling = ProjectPaths::resolve_path_from(
        &product_directory,
        Path::new(platform_runtime_library_name()),
    )
    .map_err(|error| {
        RuntimeLibraryError::new(format!(
            "could not resolve default sibling runtime library path: {error}"
        ))
    })?;
    if sibling.operation_path().exists() {
        return Ok(sibling.into_operation_path());
    }

    let deps = ProjectPaths::resolve_path_from(
        &product_directory,
        Path::new("deps").join(platform_runtime_library_name()),
    )
    .map_err(|error| {
        RuntimeLibraryError::new(format!(
            "could not resolve default dependency runtime library path: {error}"
        ))
    })?;
    if deps.operation_path().exists() {
        return Ok(deps.into_operation_path());
    }

    Ok(sibling.into_operation_path())
}

fn env_runtime_library_path() -> Result<Option<RuntimeLibraryPathSelection>, RuntimeLibraryError> {
    runtime_library_override_path_from_value(env::var_os(ZIRCON_RUNTIME_LIBRARY_ENV))
}

fn runtime_library_override_path_from_value(
    value: Option<OsString>,
) -> Result<Option<RuntimeLibraryPathSelection>, RuntimeLibraryError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    if value.to_str().is_some_and(|value| value.trim().is_empty()) {
        return Err(RuntimeLibraryError::new(format!(
            "runtime startup diagnostic: component=runtime_library requested_path=<environment override> cause={ZIRCON_RUNTIME_LIBRARY_ENV} is blank recovery=unset {ZIRCON_RUNTIME_LIBRARY_ENV} or set it to a compatible product-relative or absolute path"
        )));
    }
    let path = PathBuf::from(value);
    let request = runtime_library_environment_override_request(&path);
    if path.is_absolute() {
        return Ok(Some(RuntimeLibraryPathSelection::EnvironmentOverride {
            path,
            request,
        }));
    }
    let executable = env::current_exe().map_err(|error| {
        RuntimeLibraryError::new(format!(
            "runtime startup diagnostic: component=runtime_library requested_path={} cause=failed to resolve current executable for product-relative {ZIRCON_RUNTIME_LIBRARY_ENV}: {error} recovery=unset {ZIRCON_RUNTIME_LIBRARY_ENV} or set it to a compatible product-relative or absolute path",
            runtime_library_environment_override_request(&path)
        ))
    })?;
    runtime_library_override_path_from_executable(&path, &executable)
        .map(|path| Some(RuntimeLibraryPathSelection::EnvironmentOverride { path, request }))
}

/// Resolves an override relative to the product directory, never the launch directory.
///
/// The `ProjectPaths` resolver retains the physical operation path and owns rooted and
/// drive-relative validation, so runtime loading follows the same portable path boundary as
/// project and capture inputs without reproducing platform rules here.
fn runtime_library_override_path_from_executable(
    path: &Path,
    executable: &Path,
) -> Result<PathBuf, RuntimeLibraryError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let directory = executable.parent().ok_or_else(|| {
        RuntimeLibraryError::new(format!(
            "runtime startup diagnostic: component=runtime_library requested_path={} cause=product executable has no parent directory recovery=unset {ZIRCON_RUNTIME_LIBRARY_ENV} or set it to a compatible product-relative or absolute path",
            runtime_library_environment_override_request(path)
        ))
    })?;
    let product_directory = ProjectPaths::resolve_path(directory).map_err(|error| {
        RuntimeLibraryError::new(format!(
            "runtime startup diagnostic: component=runtime_library requested_path={} cause=could not resolve product executable directory: {error} recovery=unset {ZIRCON_RUNTIME_LIBRARY_ENV} or set it to a compatible product-relative or absolute path",
            runtime_library_environment_override_request(path)
        ))
    })?;
    ProjectPaths::resolve_path_from(&product_directory, path)
        .map(|resolved| resolved.into_operation_path())
        .map_err(|error| {
            RuntimeLibraryError::new(format!(
                "runtime startup diagnostic: component=runtime_library requested_path={} cause=could not resolve product-relative {ZIRCON_RUNTIME_LIBRARY_ENV}: {error} recovery=unset {ZIRCON_RUNTIME_LIBRARY_ENV} or set it to a compatible product-relative or absolute path",
                runtime_library_environment_override_request(path)
            ))
        })
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
    use std::path::{Path, PathBuf};

    use super::super::RuntimeLibraryError;
    use super::{
        runtime_library_environment_override_request,
        runtime_library_override_path_from_executable, runtime_library_override_path_from_value,
        RuntimeLibraryPathError, RuntimeLibraryPathSelection,
    };

    #[test]
    fn runtime_library_override_path_rejects_blank_unicode_value() {
        let error = runtime_library_override_path_from_value(Some(OsString::from("\u{2003}")))
            .expect_err("blank runtime override must not be treated as a loadable path");

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_library requested_path=<environment override> cause=ZIRCON_RUNTIME_LIBRARY is blank recovery=unset ZIRCON_RUNTIME_LIBRARY or set it to a compatible product-relative or absolute path"
        );
    }

    #[test]
    fn runtime_library_override_path_preserves_empty_fallback_and_absolute_paths() {
        let absolute = std::env::temp_dir().join("zircon-runtime-library-test.dll");

        assert_eq!(
            runtime_library_override_path_from_value(Some(OsString::new())).unwrap(),
            None
        );
        let selected =
            runtime_library_override_path_from_value(Some(absolute.clone().into_os_string()))
                .unwrap()
                .expect("an absolute override must select a runtime library");
        assert_eq!(selected.path(), absolute);
    }

    #[test]
    fn runtime_library_override_selection_keeps_a_relative_request_with_its_operation_path() {
        let relative = Path::new("plugins/zircon_runtime.dll");
        let selected = runtime_library_override_path_from_value(Some(OsString::from(
            "plugins/zircon_runtime.dll",
        )))
        .unwrap()
        .expect("a normal relative override must select a runtime library");

        assert_eq!(
            selected.environment_override_request(),
            Some("ZIRCON_RUNTIME_LIBRARY=plugins/zircon_runtime.dll")
        );

        let executable = std::env::current_exe().unwrap();
        let product_directory = zircon_runtime::asset::project::ProjectPaths::resolve_path(
            executable
                .parent()
                .expect("test executable must have a product directory"),
        )
        .unwrap();
        let expected = zircon_runtime::asset::project::ProjectPaths::resolve_path_from(
            &product_directory,
            relative,
        )
        .unwrap()
        .into_operation_path();
        assert_eq!(selected.path(), expected);
    }

    #[test]
    fn runtime_library_override_path_resolves_relative_value_from_product_directory() {
        let executable = std::env::temp_dir()
            .join("zircon-runtime-library-distribution")
            .join("zircon_runtime.exe");

        let actual = runtime_library_override_path_from_executable(
            Path::new("plugins/zircon_runtime.dll"),
            &executable,
        )
        .expect("a normal relative override should resolve from the product directory");

        let expected = zircon_runtime::asset::project::ProjectPaths::resolve_path(
            executable
                .parent()
                .expect("test executable must have a distribution directory")
                .join("plugins/zircon_runtime.dll"),
        )
        .unwrap()
        .into_operation_path();
        assert_eq!(actual, expected);
    }

    #[cfg(windows)]
    #[test]
    fn runtime_library_override_path_preserves_windows_absolute_path_semantics() {
        for absolute in [
            PathBuf::from(r"C:\zircon\zircon_runtime.dll"),
            PathBuf::from(r"\\server\share\zircon_runtime.dll"),
        ] {
            let selected =
                runtime_library_override_path_from_value(Some(absolute.clone().into_os_string()))
                    .unwrap()
                    .expect("an absolute Windows override must select a runtime library");
            assert_eq!(selected.path(), absolute);
        }

        let executable = Path::new(r"C:\zircon\zircon_runtime.exe");
        for relative in [
            PathBuf::from(r"C:zircon_runtime.dll"),
            PathBuf::from(r"\zircon_runtime.dll"),
            PathBuf::from(r"/zircon_runtime.dll"),
        ] {
            let error = runtime_library_override_path_from_executable(&relative, executable)
                .expect_err("Windows drive-relative and rooted paths must remain unsupported");

            assert!(
                error
                    .to_string()
                    .contains(&runtime_library_environment_override_request(&relative)),
                "relative path diagnostic must retain the original requested value: {error}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn runtime_library_override_path_preserves_non_utf8_value() {
        use std::os::unix::ffi::OsStringExt;

        let value = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xFF]);

        let selected = runtime_library_override_path_from_value(Some(value.clone()))
            .unwrap()
            .expect("an absolute non-UTF-8 override must select a runtime library");
        assert_eq!(selected.path(), PathBuf::from(value));
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
        let path = std::env::temp_dir()
            .join("zircon-runtime-library-product")
            .join("plugins")
            .join("custom.dll");
        let request = runtime_library_environment_override_request(Path::new("plugins/custom.dll"));
        let selection = RuntimeLibraryPathSelection::EnvironmentOverride {
            path: path.clone(),
            request: request.clone(),
        };

        assert_eq!(selection.path(), path);
        assert_eq!(
            selection.environment_override_request(),
            Some(request.as_str())
        );
    }
}
