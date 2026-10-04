use super::*;

#[test]
fn native_live_host_bridge_method_bindings_recover_poisoned_lock() {
    let host = NativePluginLiveHost::default();
    let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _bindings = host.runtime_bridge_method_bindings.lock().unwrap();
        panic!("poison native live-host bridge method bindings");
    }));
    assert!(poison.is_err());

    assert!(!host
        .clear_runtime_bridge_method_bindings("physics")
        .expect("poisoned binding lock should recover for clear"));
    assert!(matches!(
        host.installed_runtime_bridge_method_binding_count("physics"),
        Err(message) if message == "runtime plugin physics has no installed native bridge method bindings"
    ));
}
