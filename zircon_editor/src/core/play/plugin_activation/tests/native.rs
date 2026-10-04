use super::*;

#[test]
fn play_native_admission_uses_the_client_runtime_target() {
    let target = play_native_plugin_target();

    assert_eq!(target.runtime_mode, RuntimeTargetMode::ClientRuntime);
    assert_eq!(target.platform, ExportTargetPlatform::Windows);
}

#[test]
fn project_play_fails_closed_without_the_installed_admission_resolver() {
    let activation = NativePluginBridgeActivation::new(NativePluginHostHandle::default());

    let error = activation
        .activate(Some(Path::new("project")))
        .expect_err("project DLLs must not load without installed proof admission");

    assert!(error.contains("native plugin admission resolver is unavailable"));
    assert!(activation
        .active_snapshot
        .lock()
        .expect("activation snapshot lock")
        .is_none());
}
