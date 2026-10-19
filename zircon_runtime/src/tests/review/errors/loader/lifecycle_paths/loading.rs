//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f5_native_live_host_loading_uses_typed_error() {
    let loading = include_str!("../../../../../plugin/loader/live/loading.rs");
    let bridge_methods = include_str!("../../../../../plugin/loader/live/bridge_methods.rs");
    let registration_replay =
        include_str!("../../../../../plugin/loader/live/registration_replay.rs");
    let runtime_behavior = include_str!("../../../../../plugin/loader/live/runtime_behavior.rs");
    let live_host_root = include_str!("../../../../../plugin/loader/live.rs");
    let live_host_tests = include_str!("../../../../../plugin/loader/live/tests/cases.rs");
    let _native_boundary =
        include_str!("../../../../../../../docs/architecture/native-plugin-boundary.md");
    let _review_findings = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let _runtime_15_plan = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md");
    let _runtime_index = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let _convention =
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let _module_convention = include_str!(
        "../../../../../../../docs/crates/zircon_runtime/structure/module-convention.md"
    );

    for required in [
        "type NativePluginLiveHostLoadingResult<T>",
        "std::result::Result<T, NativePluginLiveHostLoadingError>",
        "enum NativePluginLiveHostLoadingError",
        "LiveHostLockPoisoned",
        "UnloadBeforeReload",
        "RuntimeBridgeMethodBindings",
        "impl std::fmt::Display for NativePluginLiveHostLoadingError",
        "impl std::error::Error for NativePluginLiveHostLoadingError",
        ") -> NativePluginLiveHostLoadingResult<NativePluginLiveHostLoadReport>",
        ") -> NativePluginLiveHostLoadingResult<MutexGuard<'_, NativePluginLiveRegistry<LoadedNativePlugin>>>",
        "NativePluginLiveHostLoadingError::LiveHostLockPoisoned",
        "NativePluginLiveHostLoadingError::UnloadBeforeReload",
        "NativePluginLiveHostLoadingError::RuntimeBridgeMethodBindings",
    ] {
        assert!(
            loading.contains(required),
            "native live-host loading typed-error owner should contain `{required}`"
        );
    }

    let production = production_section(loading);
    for forbidden in [
        ") -> Result<MutexGuard<'_, BTreeMap<String, LoadedNativePlugin>>, String>",
        ".map_err(|_| \"native plugin live host lock is poisoned\".to_string())",
        "discovered_runtime_bridge_method_bindings(&plugin)",
        "self.replace_runtime_bridge_method_bindings(&plugin_id, bindings)?",
        ") -> Result<MutexGuard<'_, NativePluginLiveRegistry<LoadedNativePlugin>>, String>",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host loading owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    for required in [
        "loaded: &ObservedLoadedNativePlugins,",
        "Err(error) => {\n                    existing.cancel_lifecycle_transition();\n                    return Err(error);\n                }",
    ] {
        assert!(loading.contains(required), "current plugin/loader/live/loading.rs must retain {required}");
    }
    for required_source in [
        "lock_loaded_native_plugins, NativePluginLiveHostLoadingError",
        "LiveHostLock(NativePluginLiveHostLoadingError)",
        "Self::LiveHostLock(",
        ") => Some(",
    ] {
        assert!(
            bridge_methods.contains(required_source)
                || registration_replay.contains(required_source)
                || runtime_behavior.contains(required_source),
            "native live-host typed-error wrappers should preserve loading source via `{required_source}`"
        );
    }
    assert!(
        live_host_root.contains("NativePluginLiveHostLoadingError"),
        "native live-host test root should expose the typed loading error to module-local tests"
    );

    for required_test in [
        "native_live_host_loading_lock_reports_typed_error",
        "NativePluginLiveHostLoadingError::LiveHostLockPoisoned",
    ] {
        assert!(
            live_host_tests.contains(required_test),
            "native live-host loading tests should contain `{required_test}`"
        );
    }
}
