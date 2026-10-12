use super::*;

#[test]
fn streaming_hot_reload_diagnostics_preserve_contract() {
    let error = NativePluginHotReloadError::RestoreRuntimeState {
        plugin_id: "physics".to_string(),
        status_code: 17,
        diagnostics: vec!["first".to_string(), "second".to_string()],
    };
    assert_eq!(
        error.to_string(),
        "plugin physics hot reload failed while restoring runtime state: status 17; first; second"
    );

    let empty = NativePluginHotReloadError::RestoreRuntimeState {
        plugin_id: "physics".to_string(),
        status_code: 18,
        diagnostics: Vec::new(),
    };
    assert_eq!(
        empty.to_string(),
        "plugin physics hot reload failed while restoring runtime state: status 18; "
    );

    let mut state =
        NativePluginHotReloadState::new(PluginModuleKind::Runtime, "physics".to_string(), None);
    state.mark_existing_unloaded(vec!["first".to_string(), "second".to_string()]);
    let rollback = state.rollback_diagnostic();
    assert_eq!(
        rollback,
        "rollback unavailable because previous runtime native package was already unloaded; first; second"
    );
    assert_eq!(rollback.len(), rollback.capacity());
    assert_eq!(
        state.rollback_error("reload failed".to_string()),
        "reload failed; rollback unavailable because previous runtime native package was already unloaded; first; second"
    );
}
