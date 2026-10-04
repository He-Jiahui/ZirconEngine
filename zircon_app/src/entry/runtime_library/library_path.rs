//! 按产品可执行文件位置选择 Runtime 动态库，并保留显式覆盖与默认发现的来源。
//! 选择结果只是一条路径；构建身份和 ABI 必须由后续加载端验证。

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
/// 将路径与请求来源一起交给加载器，使错误能指出用户覆盖还是默认发现。
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

/// 供预检和真正加载共用，避免两个阶段采用不同的动态库选择规则。
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

/// 默认产物优先采用可执行文件同级目录，其次采用依赖目录；缺失留给后续验证报告。
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
#[path = "tests/library_path.rs"]
mod tests;
