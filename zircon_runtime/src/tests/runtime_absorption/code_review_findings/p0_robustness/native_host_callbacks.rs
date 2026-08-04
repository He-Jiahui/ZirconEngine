#[test]
fn review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi() {
    let panic_guard = include_str!("../../../../plugin/native_plugin_loader/ffi_panic_guard.rs");
    let host_api_adapter =
        include_str!("../../../../plugin/native_plugin_loader/host_api_adapter.rs");
    let host_callbacks = include_str!("../../../../plugin/native_plugin_loader/host_callbacks.rs");
    let native_loader_doc =
        include_str!("../../../../../../docs/zircon_runtime/plugin/native_plugin_loader/index.md");
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");

    for required in [
        "catch_unwind(AssertUnwindSafe(call))",
        "catch_native_host_api_panic(call: impl FnOnce() -> ZrStatus) -> ZrStatus",
        "ZrStatusCode::Panic",
        "catch_native_plugin_host_callback_panic(call: impl FnOnce() -> u32) -> u32",
        "ZIRCON_NATIVE_PLUGIN_STATUS_PANIC",
    ] {
        assert!(
            panic_guard.contains(required),
            "native loader panic guard should contain `{required}`"
        );
    }

    for callback in [
        "native_host_register_system_v1",
        "native_host_register_component_v1",
        "native_host_spawn_command_v1",
        "native_host_asset_request_v1",
        "native_host_event_emit_v1",
        "native_host_event_drain_v1",
        "native_host_bridge_call_v1",
        "native_host_diagnostics_emit_v1",
        "native_host_diagnostics_metric_v1",
    ] {
        let callback_start = host_api_adapter
            .find(&format!("unsafe extern \"C\" fn {callback}"))
            .unwrap_or_else(|| panic!("missing public native host API callback `{callback}`"));
        let callback_body = &host_api_adapter[callback_start..];
        assert!(
            callback_body
                .lines()
                .take(12)
                .any(|line| line.contains("catch_native_host_api_panic")),
            "`{callback}` should enter catch_native_host_api_panic before host logic"
        );
    }

    for callback in [
        "native_host_abi_version_v3",
        "native_host_has_capability_v3",
        "native_host_log_v3",
        "native_host_diagnostic_v3",
    ] {
        let callback_start = host_callbacks
            .find(&format!("unsafe extern \"C\" fn {callback}"))
            .unwrap_or_else(|| panic!("missing private native host callback `{callback}`"));
        let callback_body = &host_callbacks[callback_start..];
        assert!(
            callback_body
                .lines()
                .take(8)
                .any(|line| line.contains("catch_native_plugin_host_callback_panic")),
            "`{callback}` should enter catch_native_plugin_host_callback_panic before host logic"
        );
    }

    let f1_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F1 |"))
        .expect("F1 row should exist");
    assert!(
        f1_row.ends_with("| Runtime 15 + Runtime 06 + Plugins 11 / review closed |"),
        "F1 row should mark the panic-guard review state closed"
    );
}
