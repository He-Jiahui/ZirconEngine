use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::*;
use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
use crate::core::framework::platform::{
    PreferenceDurabilityState, PreferenceKey, PreferenceKeyErrorKind, PreferenceMutationTerminal,
    PreferenceStorage, PreferenceStorageBackendKind, PreferenceStorageErrorKind,
    PreferenceTicketWaitResult, PreferenceWorkDeadline,
};
use crate::core::manager::{
    platform_preference_storage_handle, resolve_manager_service, PLATFORM_MANAGER_NAME,
};
use crate::core::{CoreError, CoreRuntime};
use crate::foundation as foundation_runtime;
use crate::platform::test_support::{
    platform_driver, platform_manager, platform_manager_with_backend, TestPlatformManager,
};

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn platform_module_driver_uses_the_activating_runtime_io_owner() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_runtime::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(crate::core::framework::platform::PLATFORM_MODULE_NAME)
        .unwrap();

    let driver = runtime
        .resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)
        .unwrap();

    assert!(driver.preference_persistence_uses_task_pool(runtime.task_graph().worker_pool()));
}

#[test]
fn platform_preference_storage_keys_require_non_empty_bounded_namespaces_and_keys() {
    let key = PreferenceKey::new("woc.input", "keybinds:account-a").expect("valid key");
    assert_eq!(key.namespace(), "woc.input");
    assert_eq!(key.key(), "keybinds:account-a");

    assert_eq!(
        PreferenceKey::new("", "key").unwrap_err().kind(),
        PreferenceKeyErrorKind::EmptyNamespace
    );
    assert_eq!(
        PreferenceKey::new("woc.input", "").unwrap_err().kind(),
        PreferenceKeyErrorKind::EmptyKey
    );
    assert_eq!(
        PreferenceKey::new("woc\0input", "key").unwrap_err().kind(),
        PreferenceKeyErrorKind::ContainsNul
    );
}

#[test]
fn platform_preference_storage_unavailable_backend_never_falls_back_to_process_memory() {
    let storage = platform_manager();
    let key = PreferenceKey::new("woc.input", "keybinds").unwrap();
    let submission = storage
        .submit_write(
            key.clone(),
            Arc::from(&br#"{"jump":"space"}"#[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();

    let PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(failure)) =
        wait_ticket(&submission)
    else {
        panic!("unavailable backend should publish a failed terminal");
    };
    assert_eq!(failure.kind(), PreferenceStorageErrorKind::Unavailable);
    let snapshot = storage.snapshot(&key).unwrap();
    assert_eq!(snapshot.value(), Some(&br#"{"jump":"space"}"#[..]));
    assert_eq!(
        snapshot.durability(),
        PreferenceDurabilityState::VisibleNotDurable
    );
}

#[test]
fn platform_preference_storage_module_cleanup_is_bounded_and_preserves_services_on_timeout() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_runtime::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(crate::core::framework::platform::PLATFORM_MODULE_NAME)
        .unwrap();

    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let driver = runtime
        .resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)
        .unwrap();
    driver
        .install_preference_storage_backend(Arc::new(BlockingPreferenceStorageBackend {
            started: Mutex::new(Some(started_tx)),
            release: Mutex::new(Some(release_rx)),
        }))
        .unwrap();
    let storage = resolve_manager_service(
        &runtime.handle(),
        platform_preference_storage_handle(&runtime.handle()).unwrap(),
    )
    .unwrap();
    let submission = storage
        .submit_write(
            PreferenceKey::new("woc.input", "blocked-cleanup").unwrap(),
            Arc::from(&b"value"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    started_rx.recv().unwrap();

    let cleanup_started = Instant::now();
    let error = runtime
        .deactivate_module(crate::core::framework::platform::PLATFORM_MODULE_NAME)
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::ModuleCleanupTimeout {
            ref module,
            ref operation,
            incomplete_entries: 1,
            ..
        } if module == crate::core::framework::platform::PLATFORM_MODULE_NAME
            && operation == "preference_persistence"
    ));
    assert!(cleanup_started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        resolve_manager_service(
            &runtime.handle(),
            platform_preference_storage_handle(&runtime.handle()).unwrap(),
        )
        .unwrap()
        .backend_kind(),
        PreferenceStorageBackendKind::HostProvided
    );

    release_tx.send(()).unwrap();
    assert!(matches!(
        wait_ticket(&submission),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    runtime
        .deactivate_module(crate::core::framework::platform::PLATFORM_MODULE_NAME)
        .unwrap();
}

#[test]
fn platform_preference_storage_backend_installation_is_one_shot() {
    let first_root = fresh_temp_root("one-shot-first");
    let second_root = fresh_temp_root("one-shot-second");
    let driver = platform_driver();

    driver
        .install_preference_storage_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
            first_root.clone(),
        )))
        .unwrap();
    let error = driver
        .install_preference_storage_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
            second_root.clone(),
        )))
        .unwrap_err();

    assert_eq!(
        error.kind(),
        PreferenceStorageBackendInstallErrorKind::AlreadyInstalled
    );
    assert_eq!(
        error.current_backend(),
        PreferenceStorageBackendKind::AtomicFile
    );

    let _ = fs::remove_dir_all(first_root);
    let _ = fs::remove_dir_all(second_root);
}

#[test]
fn platform_preference_storage_atomic_file_reloads_and_isolates_namespaces() {
    let root = fresh_temp_root("reload");
    let first = manager_with_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
        root.clone(),
    )));
    let account_a = PreferenceKey::new("woc.input.account-a", "keybinds").unwrap();
    let account_b = PreferenceKey::new("woc.input.account-b", "keybinds").unwrap();

    submit_write(&first, account_a.clone(), b"account-a");
    submit_write(&first, account_b.clone(), b"account-b");
    wait_fence(&first);

    let reloaded = manager_with_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
        root.clone(),
    )));
    assert_eq!(
        wait_snapshot(&reloaded, &account_a).value(),
        Some(&b"account-a"[..])
    );
    assert_eq!(
        wait_snapshot(&reloaded, &account_b).value(),
        Some(&b"account-b"[..])
    );
    assert_eq!(
        reloaded.backend_kind(),
        PreferenceStorageBackendKind::AtomicFile
    );

    let remove = reloaded
        .submit_remove(account_a.clone(), PreferenceWorkDeadline::none())
        .unwrap();
    assert!(matches!(
        wait_ticket(&remove),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert_eq!(reloaded.snapshot(&account_a).unwrap().value(), None);
    assert_eq!(
        reloaded.snapshot(&account_b).unwrap().value(),
        Some(&b"account-b"[..])
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn platform_preference_storage_atomic_file_supports_maximum_length_keys() {
    let root = fresh_temp_root("maximum-key");
    let storage = manager_with_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
        root.clone(),
    )));
    let key = PreferenceKey::new("n".repeat(128), "k".repeat(512)).unwrap();

    submit_write(&storage, key.clone(), b"maximum-length-key");
    assert_eq!(
        storage.snapshot(&key).unwrap().value(),
        Some(&b"maximum-length-key"[..])
    );

    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn platform_preference_storage_atomic_file_supports_long_managed_temp_roots() {
    let case = format!("long-root-{}", "padding".repeat(12));
    let root = fresh_temp_root(&case);
    let storage = manager_with_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
        root.clone(),
    )));
    let key = PreferenceKey::new("woc.input", "long-managed-temp-root").unwrap();

    submit_write(&storage, key.clone(), b"long-managed-temp-root");
    assert_eq!(
        storage.snapshot(&key).unwrap().value(),
        Some(&b"long-managed-temp-root"[..])
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn platform_preference_storage_atomic_file_reports_path_stage_and_fsync_work() {
    let root = fresh_temp_root("diagnostics");
    let backend = Arc::new(AtomicFilePreferenceStorageBackend::new(root.clone()));
    let storage = manager_with_backend(backend.clone());
    let key = PreferenceKey::new("woc.input", "diagnostics").unwrap();

    submit_write(&storage, key, b"diagnostics");
    let diagnostics = PreferenceStorageBackend::diagnostics(backend.as_ref());
    assert_eq!(diagnostics.writes, 1);
    assert!(diagnostics.path_build_wall > Duration::ZERO);
    assert!(diagnostics.staged_write_wall > Duration::ZERO);
    assert!(diagnostics.fsync_wall > Duration::ZERO);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn platform_preference_storage_manager_handle_resolves_the_injected_contract() {
    let root = fresh_temp_root("manager-handle");
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_runtime::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(crate::core::framework::platform::PLATFORM_MODULE_NAME)
        .unwrap();

    let driver = runtime
        .resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)
        .unwrap();
    driver
        .install_preference_storage_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
            root.clone(),
        )))
        .unwrap();

    let handle = platform_preference_storage_handle(&runtime.handle()).unwrap();
    assert_eq!(handle.service.as_str(), PLATFORM_MANAGER_NAME);
    let storage = resolve_manager_service(&runtime.handle(), handle).unwrap();
    let key = PreferenceKey::new("woc.input", "gamepad").unwrap();
    submit_write(storage.as_ref(), key.clone(), b"button-map");
    assert_eq!(
        storage.snapshot(&key).unwrap().value(),
        Some(&b"button-map"[..])
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn platform_preference_storage_injected_backend_updates_capability_report() {
    let root = fresh_temp_root("capability");
    let manager = manager_with_backend(Arc::new(AtomicFilePreferenceStorageBackend::new(
        root.clone(),
    )));
    let config = PlatformConfig {
        enabled: true,
        target: PlatformTarget::Windows,
        target_mode: crate::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        features: PlatformFeatureSelection::bevy_default_platform(),
    };

    let report = manager.planning_capability_report(&config);
    assert_eq!(
        report.persistent_preferences,
        CapabilityStatus::Supported(PreferenceStorageBackendKind::AtomicFile)
    );
    assert!(report
        .diagnostic_lines()
        .contains(&"platform.persistent_preferences=supported:atomic_file".to_owned()));

    let _ = fs::remove_dir_all(root);
}

fn manager_with_backend(backend: Arc<dyn PreferenceStorageBackend>) -> TestPlatformManager {
    platform_manager_with_backend(backend)
}

fn submit_write(storage: &dyn PreferenceStorage, key: PreferenceKey, value: &[u8]) {
    let submission = storage
        .submit_write(key, Arc::from(value), PreferenceWorkDeadline::none())
        .unwrap();
    let terminal = wait_ticket(&submission);
    assert!(
        matches!(
            &terminal,
            PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
        ),
        "preference write ended as {terminal:?}"
    );
}

fn wait_ticket(
    submission: &crate::core::framework::platform::PreferenceMutationSubmission,
) -> PreferenceTicketWaitResult {
    submission
        .ticket()
        .wait_until(Instant::now() + Duration::from_secs(2))
}

fn wait_fence(storage: &dyn PreferenceStorage) {
    let fence = storage.flush_fence(PreferenceWorkDeadline::none()).unwrap();
    assert!(matches!(
        fence.wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
}

fn wait_snapshot(
    storage: &dyn PreferenceStorage,
    key: &PreferenceKey,
) -> crate::core::framework::platform::PreferenceReadSnapshot {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let snapshot = storage.snapshot(key).unwrap();
        if snapshot.durability() != PreferenceDurabilityState::Pending {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "preference snapshot did not load"
        );
        std::thread::yield_now();
    }
}

fn fresh_temp_root(case: &str) -> PathBuf {
    let id = NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed);
    // Keep routine fixtures compact; the dedicated long-root regression covers verbatim paths.
    let root = std::env::temp_dir().join(format!("zr-pref-{case}-{}-{id}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}

struct BlockingPreferenceStorageBackend {
    started: Mutex<Option<mpsc::SyncSender<()>>>,
    release: Mutex<Option<mpsc::Receiver<()>>>,
}

impl PreferenceStorageBackend for BlockingPreferenceStorageBackend {
    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        PreferenceStorageBackendKind::HostProvided
    }

    fn open_read(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
    ) -> Result<
        Option<Box<dyn Read + Send>>,
        crate::core::framework::platform::PreferenceStorageError,
    > {
        Ok(None)
    }

    fn write(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
        _value: &[u8],
    ) -> Result<(), crate::core::framework::platform::PreferenceStorageError> {
        if let Some(started) = self.started.lock().unwrap().take() {
            started.send(()).unwrap();
        }
        if let Some(release) = self.release.lock().unwrap().take() {
            release.recv().unwrap();
        }
        Ok(())
    }

    fn remove(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        _key: &PreferenceKey,
    ) -> Result<(), crate::core::framework::platform::PreferenceStorageError> {
        Ok(())
    }

    fn flush(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), crate::core::framework::platform::PreferenceStorageError> {
        Ok(())
    }
}
