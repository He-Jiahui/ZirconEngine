//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_f5_native_live_host_bridge_lifecycle_uses_typed_error() {
    let bridge_lifecycle = include_str!("../../../../../plugin/loader/live/bridge_lifecycle.rs");
    let live_host_root = include_str!("../../../../../plugin/loader/live.rs");
    let live_host_tests = include_str!("../../../../../plugin/loader/live/tests/cases.rs");
    let _native_boundary =
        include_str!("../../../../../../../docs/architecture/native-plugin-boundary.md");
    let _module_convention = include_str!(
        "../../../../../../../docs/crates/zircon_runtime/structure/module-convention.md"
    );

    for required in [
        "type NativePluginBridgeLifecycleResult<T>",
        "std::result::Result<T, NativePluginBridgeLifecycleError>",
        "enum NativePluginBridgeLifecycleError",
        "Load",
        "HotReload",
        "Unload",
        "BridgeLifecycleRejected",
        "UnloadRollback",
        "impl std::fmt::Display for NativePluginBridgeLifecycleError",
        "impl std::error::Error for NativePluginBridgeLifecycleError",
        ") -> NativePluginBridgeLifecycleResult<NativePluginRuntimeHotUpdateReport>",
        ") -> NativePluginBridgeLifecycleResult<NativePluginLiveHostLoadReport>",
        ") -> NativePluginBridgeLifecycleResult<NativePluginLiveHostOutcome>",
        "NativePluginBridgeLifecycleError::BridgeLifecycleRejected",
        "NativePluginBridgeLifecycleError::UnloadRollback",
    ] {
        assert!(
            bridge_lifecycle.contains(required),
            "native live-host bridge-lifecycle typed-error owner should contain `{required}`"
        );
    }

    let production = bridge_lifecycle
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host bridge-lifecycle production source");
    for forbidden in [
        "return Err(bridge_report.diagnostic());",
        "Err(format!(\"{error}; {}\", rollback_report.diagnostic()))",
        "let mut report = self.hot_reload_runtime_plugins_from_export_root(export_root)?;",
        "let mut report = self.load_runtime_plugins_from_export_root(export_root)?;",
        "let mut outcome = self.hot_reload_runtime_plugin(root, plugin_id)?;",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host bridge-lifecycle owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        bridge_lifecycle.contains(".map_err(|error| error.to_string())"),
        "native live-host bridge-lifecycle public APIs should stringify typed errors only at public boundaries"
    );
    assert!(
        live_host_root.contains("NativePluginBridgeLifecycleError"),
        "native live-host test root should expose the typed bridge-lifecycle error to module-local tests"
    );

    for required_test in [
        "native_live_host_bridge_lifecycle_rejected_unload_reports_typed_error",
        "NativePluginBridgeLifecycleError::BridgeLifecycleRejected",
    ] {
        assert!(
            live_host_tests.contains(required_test),
            "native live-host bridge-lifecycle tests should contain `{required_test}`"
        );
    }
}
