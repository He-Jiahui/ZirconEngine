use super::*;

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
