use super::runtime_src_path;
use super::rust_source_view::{production_code_view, production_section};
use super::support::assert_contains_all_exact;

const LOCK_UNWRAP_CALL: &str = concat!(".lock().", "unwrap()");

#[test]
fn runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry() {
    let bridge_methods =
        read_runtime_src("plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs");
    let structure_parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");

    assert_contains_all_exact(
        "native live host bridge methods poison recovery",
        &bridge_methods,
        &[
            "use std::collections::HashMap;",
            "use std::sync::{Arc, MutexGuard};",
            "fn lock_runtime_bridge_method_bindings(",
            "MutexGuard<'_, NativePluginLiveRegistry<ValidatedRuntimeBridgeMethodBindings>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_runtime_bridge_method_bindings()",
            "native_live_host_bridge_method_bindings_recover_poisoned_lock",
        ],
    );
    assert_contains_all_exact(
        "native live host lock poison guard mount",
        &structure_parent,
        &[
            "#[path = \"structure_convention/native_live_host_lock_poison.rs\"]",
            "mod native_live_host_lock_poison;",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("native live host bridge methods", &bridge_methods);
    assert!(
        !production_section(&bridge_methods).contains("lock poisoned"),
        "native live host bridge method production code should recover poisoned binding locks"
    );
}

fn assert_no_direct_lock_unwrap_in_production(label: &str, source: &str) {
    let production = production_code_view(source);
    assert!(
        !production.contains(LOCK_UNWRAP_CALL),
        "{label} production code should use poison-safe lock helpers instead of {LOCK_UNWRAP_CALL}"
    );
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}
