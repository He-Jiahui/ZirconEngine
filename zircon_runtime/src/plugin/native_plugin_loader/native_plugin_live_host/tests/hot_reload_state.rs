use super::*;

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

#[test]
fn native_live_host_rollback_plan_reports_when_previous_plugin_was_restored() {
    let existing = native_live_host_test_plugin("physics", PluginModuleKind::Runtime);
    let mut reload_state = NativePluginHotReloadState::new(
        PluginModuleKind::Runtime,
        "runtime:physics".to_string(),
        Some(existing),
    );

    let _unloaded = reload_state
        .take_existing_for_unload()
        .expect("existing plugin should be taken for unload");
    reload_state.mark_existing_unloaded(Vec::new());
    reload_state.mark_existing_restored();

    assert!(reload_state
        .rollback_diagnostic()
        .contains("rolled back to the previously loaded runtime native package"));
}

#[test]
fn native_hot_reload_state_saves_and_restores_runtime_snapshot() {
    restored_payloads().lock().unwrap().clear();
    let existing = native_live_host_test_plugin_with_behavior(
        "physics",
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 7,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: None,
            event_manifest: None,
            registration_manifest: None,
            invoke_command: None,
            save_state: Some(hot_reload_save_state),
            restore_state: Some(hot_reload_restore_state),
            unload: None,
        },
    );
    let mut reload_state = NativePluginHotReloadState::new(
        PluginModuleKind::Runtime,
        "runtime:physics".to_string(),
        Some(existing),
    );

    let snapshot = reload_state
        .save_existing_runtime_snapshot("physics")
        .expect("stateful plugin snapshot should save")
        .expect("stateful plugin should produce a snapshot")
        .clone();
    let replacement = native_live_host_test_plugin_with_behavior(
        "physics",
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 7,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: None,
            event_manifest: None,
            registration_manifest: None,
            invoke_command: None,
            save_state: None,
            restore_state: Some(hot_reload_restore_state),
            unload: None,
        },
    );

    let diagnostics =
        restore_runtime_snapshot(&snapshot, &replacement).expect("snapshot should restore");

    assert!(diagnostics.is_empty());
    assert_eq!(snapshot.blob, b"state:physics".to_vec());
    assert_eq!(
        restored_payloads().lock().unwrap().as_slice(),
        &[b"state:physics".to_vec()]
    );
}

#[test]
fn native_hot_reload_snapshot_save_reports_typed_status_error() {
    let existing = native_live_host_test_plugin_with_behavior(
        "physics",
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 7,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: None,
            event_manifest: None,
            registration_manifest: None,
            invoke_command: None,
            save_state: Some(hot_reload_save_state_failure),
            restore_state: Some(hot_reload_restore_state),
            unload: None,
        },
    );
    let mut reload_state = NativePluginHotReloadState::new(
        PluginModuleKind::Runtime,
        "runtime:physics".to_string(),
        Some(existing),
    );

    let error = reload_state
        .save_existing_runtime_snapshot("physics")
        .unwrap_err();

    assert!(matches!(
        error,
        NativePluginHotReloadError::SaveRuntimeState {
            ref plugin_id,
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
        } if plugin_id == "physics"
    ));
    assert!(error
        .to_string()
        .contains("plugin physics hot reload failed while saving runtime state"));
}

#[test]
fn native_hot_reload_snapshot_restore_rejects_schema_mismatch() {
    let existing = native_live_host_test_plugin_with_behavior(
        "physics",
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 7,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: None,
            event_manifest: None,
            registration_manifest: None,
            invoke_command: None,
            save_state: Some(hot_reload_save_state),
            restore_state: Some(hot_reload_restore_state),
            unload: None,
        },
    );
    let mut reload_state = NativePluginHotReloadState::new(
        PluginModuleKind::Runtime,
        "runtime:physics".to_string(),
        Some(existing),
    );
    let snapshot = reload_state
        .save_existing_runtime_snapshot("physics")
        .expect("stateful plugin snapshot should save")
        .expect("stateful plugin should produce a snapshot")
        .clone();
    let replacement = native_live_host_test_plugin_with_behavior(
        "physics",
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 8,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: None,
            event_manifest: None,
            registration_manifest: None,
            invoke_command: None,
            save_state: None,
            restore_state: Some(hot_reload_restore_state),
            unload: None,
        },
    );

    let error = restore_runtime_snapshot(&snapshot, &replacement).unwrap_err();

    assert!(matches!(
        error,
        NativePluginHotReloadError::StateSchemaMismatch {
            ref plugin_id,
            snapshot_schema: Some(7),
            loaded_schema: Some(8)
        } if plugin_id == "physics"
    ));
    assert!(error
        .to_string()
        .contains("snapshot state schema Some(7) does not match loaded state schema Some(8)"));
}

#[test]
fn hot_reload_failure_rolls_back_to_snapshot() {
    restored_payloads().lock().unwrap().clear();
    let existing = native_live_host_test_plugin_with_behavior(
        "physics",
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 7,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: None,
            event_manifest: None,
            registration_manifest: None,
            invoke_command: None,
            save_state: Some(hot_reload_save_state),
            restore_state: Some(hot_reload_restore_state),
            unload: None,
        },
    );
    let mut reload_state = NativePluginHotReloadState::new(
        PluginModuleKind::Runtime,
        "runtime:physics".to_string(),
        Some(existing),
    );
    let snapshot = reload_state
        .save_existing_runtime_snapshot("physics")
        .expect("stateful plugin snapshot should save")
        .expect("stateful plugin should produce a snapshot")
        .clone();
    let existing = reload_state
        .take_existing_for_unload()
        .expect("existing plugin should be held for rollback restore");
    let replacement = native_live_host_test_plugin_with_behavior(
        "physics",
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 8,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            command_manifest: None,
            event_manifest: None,
            registration_manifest: None,
            invoke_command: None,
            save_state: None,
            restore_state: Some(hot_reload_restore_state),
            unload: None,
        },
    );

    let replacement_error = restore_runtime_snapshot(&snapshot, &replacement).unwrap_err();
    let rollback_diagnostics =
        restore_runtime_snapshot(&snapshot, &existing).expect("old snapshot should restore");

    assert!(replacement_error
        .to_string()
        .contains("snapshot state schema Some(7) does not match loaded state schema Some(8)"));
    assert!(rollback_diagnostics.is_empty());
    assert_eq!(
        restored_payloads().lock().unwrap().as_slice(),
        &[b"state:physics".to_vec()]
    );
}
