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
mod tests {
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::path::PathBuf;

    use zircon_runtime::core::framework::foundation::FOUNDATION_MODULE_NAME;
    use zircon_runtime::core::framework::platform::{
        PreferenceStorageBackendKind, RuntimeTargetMode, PLATFORM_MODULE_NAME,
    };
    use zircon_runtime::core::framework::project::RuntimeProfileId;
    use zircon_runtime::core::manager::{
        platform_preference_storage_handle, resolve_manager_service,
    };
    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::{foundation, platform};

    use super::super::{BuiltinEngineEntry, EngineEntry, EntryProfile};
    use super::{
        install_preference_storage_backend, planned_preference_storage_backend,
        preference_storage_root, HostPreferenceStorageBackend,
    };

    #[test]
    fn platform_preference_storage_host_selects_desktop_user_data_roots() {
        let windows = env_map([("LOCALAPPDATA", r"C:\Users\player\AppData\Local")]);
        assert_eq!(
            preference_storage_root(platform::PlatformTarget::Windows, windows),
            Some(PathBuf::from(
                r"C:\Users\player\AppData\Local\ZirconEngine\preferences"
            ))
        );

        let linux = env_map([("XDG_DATA_HOME", "/home/player/.data")]);
        assert_eq!(
            preference_storage_root(platform::PlatformTarget::Linux, linux),
            Some(PathBuf::from("/home/player/.data/ZirconEngine/preferences"))
        );

        let macos = env_map([("HOME", "/Users/player")]);
        assert_eq!(
            preference_storage_root(platform::PlatformTarget::Macos, macos),
            Some(PathBuf::from(
                "/Users/player/Library/Application Support/ZirconEngine/preferences"
            ))
        );
    }

    #[test]
    fn platform_preference_storage_host_requires_mobile_browser_injection() {
        for target in [
            platform::PlatformTarget::Android,
            platform::PlatformTarget::Ios,
            platform::PlatformTarget::WebGpu,
            platform::PlatformTarget::Wasm,
            platform::PlatformTarget::Headless,
        ] {
            assert_eq!(preference_storage_root(target, env_map([])), None);
        }
    }

    #[test]
    fn platform_preference_storage_host_reports_explicit_mobile_backend() {
        let backend = HostPreferenceStorageBackend::new(std::sync::Arc::new(
            platform::AtomicFilePreferenceStorageBackend::new("mobile-sandbox"),
        ));
        let config = platform::PlatformConfig {
            enabled: true,
            target: platform::PlatformTarget::Android,
            target_mode: RuntimeTargetMode::ClientRuntime,
            features: platform::PlatformFeatureSelection::bevy_default_platform(),
        };

        assert_eq!(
            planned_preference_storage_backend(&config, Some(&backend)),
            PreferenceStorageBackendKind::AtomicFile
        );
    }

    #[test]
    fn builtin_engine_entry_installs_and_reports_explicit_preference_backend() {
        let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime)
            .unwrap()
            .with_preference_storage_backend(std::sync::Arc::new(
                platform::AtomicFilePreferenceStorageBackend::new("explicit-host-preferences"),
            ));
        let report = entry.module_selection_report();

        assert_eq!(
            report.preference_storage_backend,
            PreferenceStorageBackendKind::AtomicFile
        );
        assert!(report
            .diagnostic_lines()
            .contains(&"platform.persistent_preferences=supported:atomic_file".to_owned()));

        let core = entry.bootstrap().unwrap();
        let handle = platform_preference_storage_handle(&core).unwrap();
        let storage = resolve_manager_service(&core, handle).unwrap();
        assert_eq!(
            storage.backend_kind(),
            PreferenceStorageBackendKind::AtomicFile
        );
    }

    #[test]
    fn minimal_entry_ignores_preference_backend_when_platform_is_disabled() {
        let entry = BuiltinEngineEntry::for_runtime_profile(RuntimeProfileId::Minimal)
            .unwrap()
            .with_preference_storage_backend(std::sync::Arc::new(
                platform::AtomicFilePreferenceStorageBackend::new("disabled-preferences"),
            ));
        let report = entry.module_selection_report();

        assert!(!report.platform_config.enabled);
        assert_eq!(
            report.preference_storage_backend,
            PreferenceStorageBackendKind::Unavailable
        );
        entry.bootstrap().unwrap();
    }

    #[test]
    fn platform_preference_storage_host_can_install_backend_on_manual_runtime() {
        let runtime = CoreRuntime::new();
        runtime
            .register_module(foundation::module_descriptor())
            .unwrap();
        runtime
            .register_module(platform::module_descriptor())
            .unwrap();
        runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        runtime.activate_module(PLATFORM_MODULE_NAME).unwrap();
        let root = std::env::temp_dir().join(format!(
            "zircon-app-preference-storage-{}",
            std::process::id()
        ));

        assert_eq!(
            install_preference_storage_backend(
                &runtime,
                Some(std::sync::Arc::new(
                    platform::AtomicFilePreferenceStorageBackend::new(root),
                )),
            )
            .unwrap(),
            PreferenceStorageBackendKind::AtomicFile
        );
        let handle = platform_preference_storage_handle(&runtime.handle()).unwrap();
        let storage = resolve_manager_service(&runtime.handle(), handle).unwrap();
        assert_eq!(
            storage.backend_kind(),
            PreferenceStorageBackendKind::AtomicFile
        );
    }

    #[test]
    fn builtin_bootstrap_installs_preference_backend_before_remaining_module_activation() {
        let source = include_str!("engine_entry.rs");
        let wire_factory = source
            .find("runtime.register_module(descriptor_with_preference_storage_backend(")
            .expect("bootstrap must wire the host backend into the platform driver factory");
        let activate_remaining = source[wire_factory..]
            .find("runtime.activate_registered_modules()?")
            .map(|offset| wire_factory + offset)
            .expect("bootstrap must activate registered modules after wiring factories");

        assert!(
            wire_factory < activate_remaining,
            "activation-time consumers must never observe the temporary unavailable backend"
        );
        assert!(source.contains("PlatformDriver::with_preference_storage_backend"));
    }

    fn env_map<const N: usize>(values: [(&str, &str); N]) -> impl Fn(&str) -> Option<OsString> {
        let values = values
            .into_iter()
            .map(|(key, value)| (key.to_owned(), OsString::from(value)))
            .collect::<HashMap<_, _>>();
        move |name| values.get(name).cloned()
    }
}
