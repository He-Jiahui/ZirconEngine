#[test]
fn review_f5_native_host_api_adapter_uses_typed_error() {
    let host_api_adapter =
        include_str!("../../../../../../plugin/native_plugin_loader/host_api_adapter.rs");
    let host_api_adapter_tests =
        include_str!("../../../../../../plugin/native_plugin_loader/host_api_adapter/tests.rs");
    let native_boundary =
        include_str!("../../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let review_findings =
        include_str!("../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let module_convention =
        include_str!("../../../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "type NativeHostApiAdapterResult<T>",
        "std::result::Result<T, NativeHostApiAdapterError>",
        "enum NativeHostApiAdapterError",
        "InvalidPluginModuleOwner",
        "InvalidUtf8",
        "UnknownSystemStage",
        "InvalidSystemSet",
        "RegisterSystem",
        "UnknownPluginModuleOwner",
        "RegisterComponent",
        "impl std::fmt::Display for NativeHostApiAdapterError",
        "impl std::error::Error for NativeHostApiAdapterError",
        ") -> NativeHostApiAdapterResult<Self>",
        ") -> NativeHostApiAdapterResult<()>",
        "fn stage_from_abi(stage: u32) -> NativeHostApiAdapterResult<SystemStage>",
        ") -> NativeHostApiAdapterResult<Vec<String>>",
        "unsafe fn read_utf8(slice: ZrByteSlice) -> NativeHostApiAdapterResult<String>",
        "NativeHostApiAdapterError::InvalidUtf8",
        "NativeHostApiAdapterError::RegisterSystem",
        "NativeHostApiAdapterError::RegisterComponent",
    ] {
        assert!(
            host_api_adapter.contains(required),
            "native host API adapter typed-error owner should contain `{required}`"
        );
    }

    let production = host_api_adapter
        .split("#[cfg(test)]")
        .next()
        .expect("native host API adapter production source");
    for forbidden in [
        "registry\n            .intern_plugin_module(module_name)\n            .map_err(|error| error.to_string())?",
        ") -> Result<(), String> {\n    let id = read_utf8(registration.system_id)?;",
        ") -> Result<(), String> {\n    let type_id = read_utf8(descriptor.type_id)?;",
        "fn stage_from_abi(stage: u32) -> Result<SystemStage, String>",
        ") -> Result<Vec<String>, String>",
        "unsafe fn read_utf8(slice: ZrByteSlice) -> Result<String, String>",
        ".ok_or_else(|| format!(\"unknown plugin module owner",
        ".intern_system_set(set_name)\n                .map_err(|error| error.to_string())",
        "builder.register().map_err(|error| error.to_string())",
        ".register_component(ComponentTypeDescriptor::new(\n            type_id,\n            plugin_id,\n            display_name,\n        ))\n        .map_err(|error| error.to_string())",
    ] {
        assert!(
            !production.contains(forbidden),
            "native host API adapter should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        host_api_adapter.contains("Self::new_result(registry, module_name).map_err(|error| error.to_string())")
            && host_api_adapter.contains("Err(_) => status(ZrStatusCode::Error),"),
        "native host API adapter should keep string/status diagnostics only at public construction and C ABI callback boundaries"
    );

    for required_test in [
        "native_host_api_adapter_reports_unknown_stage_with_typed_error",
        "native_host_api_adapter_utf8_error_preserves_source",
    ] {
        assert!(
            host_api_adapter_tests.contains(required_test),
            "native host API adapter tests should contain `{required_test}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 native host API adapter typed errors",
        "runtime_15_native_host_api_adapter_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_host_api_adapter_uses_typed_error",
        "plugin/native_plugin_loader/host_api_adapter.rs",
        "NativeHostApiAdapterError::InvalidUtf8",
        "host API adapter keeps string diagnostics at public construction and C ABI status boundaries",
    ] {
        assert!(
            native_boundary.contains(doc_anchor)
                || review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || module_convention.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "native host API adapter docs/status should record `{doc_anchor}`"
        );
    }
}
