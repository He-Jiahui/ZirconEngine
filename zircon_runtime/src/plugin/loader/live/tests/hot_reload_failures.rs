use super::super::super::abi_declarations::{
    NativePluginRestoreStateFnV3, NativePluginSaveStateFnV3,
};
use super::*;

#[test]
fn hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_behavior(
                "physics",
                hot_reload_stateful_behavior(
                    Some(hot_reload_save_state),
                    Some(hot_reload_restore_state),
                ),
            ),
        );
    }
    let mut report = NativePluginLoadReport::default();
    report.push_diagnostic("native plugin physics skipped because runtime entry symbol is missing");

    let error = host
        .hot_reload_reported_plugin(
            report,
            &std::path::PathBuf::from("reload-root"),
            "physics",
            PluginModuleKind::Runtime,
        )
        .unwrap_err();

    assert!(error.contains("plugin physics hot reload did not load a runtime native package"));
    assert!(error.contains("runtime entry symbol is missing"));
    assert!(error.contains("rolled back to the previously loaded runtime native package"));
    assert_eq!(
        host.loaded_plugin_ids(PluginModuleKind::Runtime).unwrap(),
        vec!["physics".to_string()]
    );
}

#[test]
fn hot_reload_state_restore_failure_rolls_back_and_reports() {
    let _fixture_guard = hot_reload_payload_fixture_lock();
    restored_payloads()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_behavior(
                "physics",
                hot_reload_stateful_behavior(
                    Some(hot_reload_save_state),
                    Some(hot_reload_restore_state),
                ),
            ),
        );
    }
    let replacement = native_live_host_test_plugin_with_behavior(
        "physics",
        hot_reload_stateful_behavior(None, Some(hot_reload_restore_state_failure)),
    );
    let report = NativePluginLoadReport::from_loaded(vec![replacement]);

    let error = host
        .hot_reload_reported_plugin(
            report,
            &std::path::PathBuf::from("reload-root"),
            "physics",
            PluginModuleKind::Runtime,
        )
        .unwrap_err();

    assert!(error.contains("plugin physics hot reload failed while restoring runtime state"));
    assert!(
        error.contains("runtime restore-state after hot reload: restore failed during hot reload")
    );
    assert!(error.contains("rolled back to the previously loaded runtime native package"));
    assert_eq!(
        restored_payloads()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_slice(),
        &[b"state:physics".to_vec(), b"state:physics".to_vec()]
    );
    assert_eq!(
        host.loaded_plugin_ids(PluginModuleKind::Runtime).unwrap(),
        vec!["physics".to_string()]
    );
    let descriptor = host
        .runtime_behavior_descriptor("physics")
        .expect("rollback should keep the previous runtime plugin loaded");
    assert_eq!(descriptor.state_schema_version, Some(7));
}

fn hot_reload_stateful_behavior(
    save_state: Option<NativePluginSaveStateFnV3>,
    restore_state: Option<NativePluginRestoreStateFnV3>,
) -> NativePluginBehavior {
    NativePluginBehavior {
        is_stateless: false,
        state_schema_version: 7,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: None,
        event_manifest: None,
        registration_manifest: None,
        command_table: None,
        invoke_command: None,
        save_state,
        restore_state,
        unload: None,
    }
}
