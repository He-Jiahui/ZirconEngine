#[test]
fn review_f5_native_live_host_runtime_behavior_uses_typed_error() {
    let runtime_behavior = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs"
    );
    let live_host_root =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_live_host.rs");
    let runtime_behavior_tests = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs"
    );
    let native_boundary = include_str!(
        "../../../../../../../../../docs/engine-architecture/native-plugin-boundary.md"
    );
    let review_findings = include_str!(
        "../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let module_convention = include_str!(
        "../../../../../../../../../docs/zircon_runtime/structure/module-convention.md"
    );

    for required in [
        "type NativePluginRuntimeBehaviorResult<T>",
        "std::result::Result<T, NativePluginRuntimeBehaviorError>",
        "enum NativePluginRuntimeBehaviorError",
        "LiveHostLock",
        "RuntimePluginNotLoaded",
        "impl std::fmt::Display for NativePluginRuntimeBehaviorError",
        "impl std::error::Error for NativePluginRuntimeBehaviorError",
        ") -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeBehaviorDescriptor>",
        ") -> NativePluginRuntimeBehaviorResult<Vec<NativePluginRuntimeBehaviorDescriptor>>",
        ") -> NativePluginRuntimeBehaviorResult<NativePluginBehaviorCallReport>",
        ") -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeCommandDispatchReport>",
        ") -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeStateSnapshot>",
        ") -> NativePluginRuntimeBehaviorResult<NativePluginRuntimeStateRestoreReport>",
        ") -> NativePluginRuntimeBehaviorResult<NativePluginRuntimePlayModeSnapshot>",
        ") -> NativePluginRuntimeBehaviorResult<NativePluginRuntimePlayModeExitReport>",
        "NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded",
        ".map_err(|error| error.to_string())",
    ] {
        assert!(
            runtime_behavior.contains(required),
            "native live-host runtime behavior typed-error owner should contain `{required}`"
        );
    }

    let production = runtime_behavior
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host runtime behavior production source");
    for forbidden in [
        "use super::diagnostics::{report_diagnostics, unloaded_plugin_error};",
        ".ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?",
        "let state_snapshot = self.save_runtime_plugin_states()?",
        "let enter_report =\n            self.dispatch_runtime_plugin_command(NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND, b\"\")?",
        "let exit_report =\n            self.dispatch_runtime_plugin_command(NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND, b\"\")?",
        "let restore_report = self.restore_runtime_plugin_states(&snapshot.state_snapshot)?",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host runtime behavior owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        live_host_root.contains("NativePluginRuntimeBehaviorError"),
        "native live-host test root should expose the typed runtime behavior error to module-local tests"
    );

    for required_test in [
        "native_live_host_runtime_behavior_reports_typed_unloaded_error",
        "NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded",
    ] {
        assert!(
            runtime_behavior_tests.contains(required_test),
            "native live-host runtime behavior tests should contain `{required_test}`"
        );
    }
}
