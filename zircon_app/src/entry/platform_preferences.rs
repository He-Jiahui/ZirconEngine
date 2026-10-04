//! App 为平台偏好存储选择宿主后端；报告和模块启动沿同一注入/桌面默认规则。

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use zircon_runtime::core::framework::platform::PreferenceStorageBackendKind;
#[cfg(test)]
use zircon_runtime::core::{CoreError, CoreRuntime};
use zircon_runtime::platform::{
    AtomicFilePreferenceStorageBackend, PlatformConfig, PlatformDriver, PlatformTarget,
    PreferenceStorageBackend, PLATFORM_DRIVER_NAME,
};

const ENGINE_DATA_DIRECTORY: &str = "ZirconEngine";
const PREFERENCE_DATA_DIRECTORY: &str = "preferences";

#[derive(Clone)]
pub(super) struct HostPreferenceStorageBackend {
    backend: Arc<dyn PreferenceStorageBackend>,
}

impl HostPreferenceStorageBackend {
    pub(super) fn new(backend: Arc<dyn PreferenceStorageBackend>) -> Self {
        Self { backend }
    }

    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        self.backend.backend_kind()
    }
}

impl std::fmt::Debug for HostPreferenceStorageBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HostPreferenceStorageBackend")
            .field("backend_kind", &self.backend_kind())
            .finish()
    }
}

/// 在模块选择诊断中预报后端类型；调用时须使用将交给 bootstrap 的同一平台配置。
pub(super) fn planned_preference_storage_backend(
    config: &PlatformConfig,
    host_backend: Option<&HostPreferenceStorageBackend>,
) -> PreferenceStorageBackendKind {
    if !config.enabled {
        PreferenceStorageBackendKind::Unavailable
    } else if let Some(host_backend) = host_backend {
        host_backend.backend_kind()
    } else if default_preference_storage_root(config.target).is_some() {
        PreferenceStorageBackendKind::AtomicFile
    } else {
        PreferenceStorageBackendKind::Unavailable
    }
}

/// 在注册 Platform 模块前固定实际后端；没有桌面用户目录时返回不可用，由宿主决定是否注入。
pub(super) fn preference_storage_backend_for_bootstrap(
    config: &PlatformConfig,
    host_backend: Option<&HostPreferenceStorageBackend>,
) -> Option<Arc<dyn PreferenceStorageBackend>> {
    if !config.enabled {
        return None;
    }
    match host_backend {
        Some(host_backend) => Some(Arc::clone(&host_backend.backend)),
        None => default_preference_storage_root(config.target).map(|root| {
            Arc::new(AtomicFilePreferenceStorageBackend::new(root))
                as Arc<dyn PreferenceStorageBackend>
        }),
    }
}

#[cfg(test)]
fn install_preference_storage_backend(
    runtime: &CoreRuntime,
    backend: Option<Arc<dyn PreferenceStorageBackend>>,
) -> Result<PreferenceStorageBackendKind, CoreError> {
    let Some(backend) = backend else {
        return Ok(PreferenceStorageBackendKind::Unavailable);
    };
    let driver = runtime.resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)?;
    let backend_kind = backend.backend_kind();
    driver
        .install_preference_storage_backend(backend)
        .map_err(|error| {
            CoreError::Initialization("platform preference storage".to_owned(), error.to_string())
        })?;
    Ok(backend_kind)
}

fn default_preference_storage_root(target: PlatformTarget) -> Option<PathBuf> {
    preference_storage_root(target, |name| std::env::var_os(name))
}

// 默认文件根只适用于桌面用户数据目录；移动、浏览器和无头宿主不得推断进程 HOME 可写。
fn preference_storage_root(
    target: PlatformTarget,
    env: impl Fn(&str) -> Option<OsString>,
) -> Option<PathBuf> {
    let base = match target {
        PlatformTarget::Windows => non_empty_env_path(&env, "LOCALAPPDATA"),
        PlatformTarget::Linux => non_empty_env_path(&env, "XDG_DATA_HOME")
            .filter(|path| path.to_string_lossy().starts_with('/'))
            .or_else(|| {
                non_empty_env_path(&env, "HOME").map(|home| home.join(".local").join("share"))
            }),
        PlatformTarget::Macos => non_empty_env_path(&env, "HOME")
            .map(|home| home.join("Library").join("Application Support")),
        PlatformTarget::Android
        | PlatformTarget::Ios
        | PlatformTarget::WebGpu
        | PlatformTarget::Wasm
        | PlatformTarget::Headless => None,
    }?;
    Some(
        base.join(ENGINE_DATA_DIRECTORY)
            .join(PREFERENCE_DATA_DIRECTORY),
    )
}

fn non_empty_env_path(env: &impl Fn(&str) -> Option<OsString>, name: &str) -> Option<PathBuf> {
    env(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
#[path = "tests/platform_preferences.rs"]
mod tests;
