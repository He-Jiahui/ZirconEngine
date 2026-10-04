use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use zircon_runtime::core::framework::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::core::framework::platform::{
    PreferenceStorageBackendKind, RuntimeTargetMode, PLATFORM_MODULE_NAME,
};
use zircon_runtime::core::framework::project::RuntimeProfileId;
use zircon_runtime::core::manager::{platform_preference_storage_handle, resolve_manager_service};
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

    let runtime = zircon_runtime::core::CoreRuntime::new();
    entry.bootstrap(&runtime).unwrap();
    let core = runtime.handle();
    {
        let handle = platform_preference_storage_handle(&core).unwrap();
        let storage = resolve_manager_service(&core, handle).unwrap();
        assert_eq!(
            storage.backend_kind(),
            PreferenceStorageBackendKind::AtomicFile
        );
    }
    drop(core);
    assert!(!runtime
        .shutdown_until(std::time::Instant::now() + std::time::Duration::from_secs(5))
        .expect("direct bootstrap must close the original Core")
        .has_in_flight_work());
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
    let source = include_str!("../engine_entry.rs");
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
    assert!(source.contains("core.task_graph().worker_pool().clone()"));
}

fn env_map<const N: usize>(values: [(&str, &str); N]) -> impl Fn(&str) -> Option<OsString> {
    let values = values
        .into_iter()
        .map(|(key, value)| (key.to_owned(), OsString::from(value)))
        .collect::<HashMap<_, _>>();
    move |name| values.get(name).cloned()
}
