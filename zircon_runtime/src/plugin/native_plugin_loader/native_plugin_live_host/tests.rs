use super::*;

use crate::plugin::{
    NativePluginBehaviorValidationReport, NativePluginDescriptor, NativePluginEntryReport,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::behavior_calls::NativePluginBehavior;

#[test]
fn native_live_host_reports_missing_editor_package_on_hot_reload() {
    let project_root = std::env::temp_dir().join(format!(
        "zircon-runtime-missing-native-live-host-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let error = NativePluginLiveHost::default()
        .hot_reload_editor_plugin(&project_root, "physics")
        .unwrap_err();
    assert!(error.contains("plugin physics hot reload did not load an editor native package"));
    assert!(error.contains("native plugin root does not exist"));
}

#[test]
fn native_live_host_reports_unloaded_plugin_by_module_kind() {
    let error = NativePluginLiveHost::default()
        .unload_runtime_plugin("physics")
        .unwrap_err();
    assert_eq!(
        error,
        "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
    );
}

#[test]
fn native_live_host_runtime_behavior_calls_report_unloaded_plugin() {
    let host = NativePluginLiveHost::default();
    let expected =
        "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package";
    assert_eq!(
        host.runtime_behavior_descriptor("physics").unwrap_err(),
        expected
    );
    assert!(host
        .runtime_behavior_descriptors()
        .expect("empty runtime live host should list no descriptors")
        .is_empty());
    assert_eq!(
        host.invoke_runtime_plugin_command("physics", "simulate", b"")
            .unwrap_err(),
        expected
    );
    assert_eq!(
        host.save_runtime_plugin_state("physics").unwrap_err(),
        expected
    );
    assert_eq!(
        host.restore_runtime_plugin_state("physics", b"")
            .unwrap_err(),
        expected
    );
}

#[test]
fn native_live_host_runtime_descriptor_includes_validation_report() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }

    let descriptor = host
        .runtime_behavior_descriptor("physics")
        .expect("loaded test plugin should return a descriptor");

    let validation = descriptor
        .validation_report
        .expect("runtime descriptor should carry validation report");
    assert_eq!(validation.plugin_id, "physics");
    assert_eq!(validation.module_kind, PluginModuleKind::Runtime);
    assert!(!validation.diagnostics.is_empty());
    assert!(validation
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("runtime behavior is missing")));
}

#[test]
fn native_live_host_runtime_broadcasts_and_snapshots_empty_when_no_plugins_loaded() {
    let host = NativePluginLiveHost::default();

    let dispatch = host
        .dispatch_runtime_plugin_command("play-mode.enter", b"{}")
        .expect("empty runtime live host should still dispatch as an empty report");
    assert_eq!(dispatch.command_name, "play-mode.enter");
    assert!(dispatch.calls.is_empty());
    assert!(dispatch.diagnostics.is_empty());
    assert!(dispatch.is_clean());
    assert_eq!(dispatch.failed_call_count(), 0);
    assert!(dispatch.combined_diagnostics().is_empty());

    let snapshot = host
        .save_runtime_plugin_states()
        .expect("empty runtime live host should still save an empty snapshot");
    assert!(snapshot.plugin_states.is_empty());
    assert!(snapshot.diagnostics.is_empty());
    assert!(snapshot.is_clean());
    assert!(snapshot.combined_diagnostics().is_empty());

    let restore = host
        .restore_runtime_plugin_states(&snapshot)
        .expect("empty runtime live host should still restore an empty snapshot");
    assert!(restore.calls.is_empty());
    assert!(restore.skipped_plugin_ids.is_empty());
    assert!(restore.diagnostics.is_empty());
    assert!(restore.is_clean());
    assert_eq!(restore.failed_call_count(), 0);
    assert!(restore.combined_diagnostics().is_empty());

    let play_snapshot = host
        .enter_runtime_play_mode()
        .expect("empty runtime live host should still enter play mode");
    assert_eq!(
        play_snapshot.enter_report.command_name,
        NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND
    );
    assert!(play_snapshot.state_snapshot.plugin_states.is_empty());
    assert!(play_snapshot.is_clean());
    assert!(play_snapshot.combined_diagnostics().is_empty());
    let play_exit = host
        .exit_runtime_play_mode(&play_snapshot)
        .expect("empty runtime live host should still exit play mode");
    assert_eq!(
        play_exit.exit_report.command_name,
        NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND
    );
    assert!(play_exit.restore_report.calls.is_empty());
    assert!(play_exit.is_clean());
    assert!(play_exit.combined_diagnostics().is_empty());
}

#[test]
fn native_live_host_runtime_snapshot_restore_reports_unloaded_plugins() {
    let host = NativePluginLiveHost::default();
    let snapshot = NativePluginRuntimeStateSnapshot {
        plugin_states: vec![NativePluginRuntimePluginState {
            plugin_id: "physics".to_string(),
            state_schema_version: Some(3),
            state: b"state".to_vec(),
        }],
        diagnostics: Vec::new(),
    };

    let restore = host
        .restore_runtime_plugin_states(&snapshot)
        .expect("unloaded plugins should be restore diagnostics, not host failures");
    assert!(restore.calls.is_empty());
    assert_eq!(restore.skipped_plugin_ids, vec!["physics".to_string()]);
    assert!(!restore.is_clean());
    assert_eq!(restore.failed_call_count(), 0);
    assert_eq!(
        restore.diagnostics,
        vec![
            "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
                .to_string()
        ]
    );
    assert_eq!(restore.combined_diagnostics(), restore.diagnostics);
}

#[test]
fn native_live_host_runtime_snapshot_restore_skips_schema_mismatch() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }
    let snapshot = NativePluginRuntimeStateSnapshot {
        plugin_states: vec![NativePluginRuntimePluginState {
            plugin_id: "physics".to_string(),
            state_schema_version: Some(3),
            state: b"state".to_vec(),
        }],
        diagnostics: Vec::new(),
    };

    let restore = host
        .restore_runtime_plugin_states(&snapshot)
        .expect("schema mismatch should be a restore diagnostic, not host failure");

    assert!(restore.calls.is_empty());
    assert_eq!(restore.skipped_plugin_ids, vec!["physics".to_string()]);
    assert_eq!(restore.failed_call_count(), 0);
    assert_eq!(
        restore.diagnostics,
        vec![
            "runtime plugin physics restore-state skipped because snapshot state schema Some(3) does not match loaded state schema None"
                .to_string()
        ]
    );
    assert!(!restore.is_clean());
}

#[test]
fn native_live_host_runtime_command_interior_nul_returns_error_report() {
    INTERIOR_NUL_INVOKE_COUNT.store(0, Ordering::SeqCst);
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_behavior(
                "physics",
                NativePluginBehavior {
                    is_stateless: true,
                    state_schema_version: 0,
                    command_manifest_schema: None,
                    event_manifest_schema: None,
                    command_manifest: Some("command=valid;payload=bytes".to_string()),
                    event_manifest: None,
                    invoke_command: Some(interior_nul_probe_invoke_command),
                    save_state: None,
                    restore_state: None,
                    unload: None,
                },
            ),
        );
    }

    let report = host
        .invoke_runtime_plugin_command("physics", "bad\0name", b"")
        .expect("loaded plugin should return behavior reports for bad command names");

    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert_eq!(
        report.diagnostics,
        vec!["native plugin command name contained an interior NUL".to_string()]
    );
    assert!(report.payload.is_none());
    assert_eq!(INTERIOR_NUL_INVOKE_COUNT.load(Ordering::SeqCst), 0);
}

#[test]
fn native_runtime_reports_synthesize_callback_status_diagnostics() {
    let failed_call = NativePluginRuntimeBehaviorCall {
        plugin_id: "physics".to_string(),
        report: NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            diagnostics: Vec::new(),
            payload: None,
        },
    };
    let dispatch = NativePluginRuntimeCommandDispatchReport {
        command_name: "simulate".to_string(),
        calls: vec![failed_call.clone()],
        diagnostics: Vec::new(),
    };
    assert!(!dispatch.is_clean());
    assert_eq!(dispatch.failed_call_count(), 1);
    assert_eq!(
        dispatch.combined_diagnostics(),
        vec!["runtime plugin physics simulate returned status 1".to_string()]
    );

    let restore = NativePluginRuntimeStateRestoreReport {
        calls: vec![failed_call],
        skipped_plugin_ids: Vec::new(),
        diagnostics: Vec::new(),
    };
    assert!(!restore.is_clean());
    assert_eq!(restore.failed_call_count(), 1);
    assert_eq!(
        restore.combined_diagnostics(),
        vec!["runtime plugin physics restore-state returned status 1".to_string()]
    );
}

#[test]
fn native_live_host_loads_runtime_export_diagnostics_without_handles() {
    let export_root = std::env::temp_dir().join(format!(
        "zircon-runtime-missing-native-live-host-export-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let report = NativePluginLiveHost::default()
        .load_runtime_plugins_from_export_root(&export_root)
        .expect("missing manifest should be reported as diagnostics, not a host failure");
    assert_eq!(report.module_kind, PluginModuleKind::Runtime);
    assert!(report.loaded_plugin_ids.is_empty());
    assert!(report.runtime_plugin_registration_reports.is_empty());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("failed to read native plugin load manifest")));
}

#[test]
fn native_live_host_treats_missing_unload_hook_as_noop_unload() {
    let report = allow_missing_unload_callback_to_drop_handle(NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec!["native plugin behavior callback unload is missing".to_string()],
        payload: None,
    });
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(
        report.diagnostics,
        vec!["native plugin behavior callback unload is missing".to_string()]
    );
}

#[test]
fn native_live_host_rollback_plan_restores_existing_plugin_when_reload_fails_before_unload() {
    let existing = native_live_host_test_plugin("physics", PluginModuleKind::Runtime);
    let mut reload_state = NativePluginHotReloadState::new(
        PluginModuleKind::Runtime,
        "runtime:physics".to_string(),
        Some(existing),
    );

    let error = reload_state.rollback_error(
        "plugin physics hot reload did not load a runtime native package".to_string(),
    );

    assert!(error.contains("rolled back to the previously loaded runtime native package"));
    assert!(
        reload_state.into_rollback_plugin().is_some(),
        "existing plugin should remain available for reinsertion after failed reload"
    );
}

#[test]
fn native_live_host_keeps_existing_runtime_handle_when_reload_finds_no_replacement() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }

    let project_root = std::env::temp_dir().join(format!(
        "zircon-runtime-rollback-native-live-host-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let error = host
        .hot_reload_runtime_plugin(&project_root, "physics")
        .unwrap_err();

    assert!(error.contains("rolled back to the previously loaded runtime native package"));
    assert_eq!(
        host.loaded_plugin_ids(PluginModuleKind::Runtime).unwrap(),
        vec!["physics".to_string()]
    );
}

#[test]
fn native_live_host_rollback_plan_reports_when_previous_plugin_was_already_unloaded() {
    let existing = native_live_host_test_plugin("physics", PluginModuleKind::Runtime);
    let mut reload_state = NativePluginHotReloadState::new(
        PluginModuleKind::Runtime,
        "runtime:physics".to_string(),
        Some(existing),
    );

    let unloaded = reload_state
        .take_existing_for_unload()
        .expect("existing plugin should be taken for unload");
    let unload_diagnostics = diagnostics_from_behavior_report(
        "runtime unload before hot reload",
        unload_behavior(&unloaded, PluginModuleKind::Runtime),
    )
    .expect("test plugin unload should be a no-op success");
    reload_state.mark_existing_unloaded(unload_diagnostics);

    let error = reload_state.rollback_error(
        "plugin physics hot reload did not load a runtime native package".to_string(),
    );

    assert!(error.contains(
        "rollback unavailable because previous runtime native package was already unloaded"
    ));
    assert!(reload_state.into_rollback_plugin().is_none());
}

fn native_live_host_test_plugin(
    plugin_id: &str,
    _module_kind: PluginModuleKind,
) -> LoadedNativePlugin {
    let descriptor = NativePluginDescriptor {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        plugin_id: plugin_id.to_string(),
        package_manifest: None,
        runtime_entry_name: None,
        editor_entry_name: None,
        requested_capabilities: Vec::new(),
    };
    LoadedNativePlugin {
        plugin_id: plugin_id.to_string(),
        library_path: std::path::PathBuf::from(format!("{plugin_id}.test.dll")),
        descriptor: Some(descriptor),
        runtime_entry_report: Some(NativePluginEntryReport {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            package_manifest: None,
            diagnostics: Vec::new(),
            negotiated_capabilities: Vec::new(),
            behavior: None,
            behavior_validation: NativePluginBehaviorValidationReport::from_behavior(
                plugin_id,
                PluginModuleKind::Runtime,
                ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
                None,
            ),
        }),
        editor_entry_report: None,
        library: this_process_library(),
    }
}

fn native_live_host_test_plugin_with_behavior(
    plugin_id: &str,
    behavior: NativePluginBehavior,
) -> LoadedNativePlugin {
    let descriptor = NativePluginDescriptor {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        plugin_id: plugin_id.to_string(),
        package_manifest: None,
        runtime_entry_name: None,
        editor_entry_name: None,
        requested_capabilities: Vec::new(),
    };
    let behavior_validation = NativePluginBehaviorValidationReport::from_behavior(
        plugin_id,
        PluginModuleKind::Runtime,
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        Some(&behavior),
    );
    LoadedNativePlugin {
        plugin_id: plugin_id.to_string(),
        library_path: std::path::PathBuf::from(format!("{plugin_id}.test.dll")),
        descriptor: Some(descriptor),
        runtime_entry_report: Some(NativePluginEntryReport {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            package_manifest: None,
            diagnostics: Vec::new(),
            negotiated_capabilities: Vec::new(),
            behavior: Some(behavior),
            behavior_validation,
        }),
        editor_entry_report: None,
        library: this_process_library(),
    }
}

static INTERIOR_NUL_INVOKE_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn interior_nul_probe_invoke_command(
    _command_name: *const std::ffi::c_char,
    _payload: super::super::abi_declarations::NativePluginByteSliceV2,
    _output: *mut super::super::abi_declarations::NativePluginOwnedByteBufferV2,
) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    INTERIOR_NUL_INVOKE_COUNT.fetch_add(1, Ordering::SeqCst);
    super::super::abi_declarations::NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

fn this_process_library() -> libloading::Library {
    #[cfg(unix)]
    {
        libloading::os::unix::Library::this().into()
    }
    #[cfg(windows)]
    {
        libloading::os::windows::Library::this()
            .expect("current process library handle should be available")
            .into()
    }
}
