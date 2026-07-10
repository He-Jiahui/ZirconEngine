#[test]
fn review_f5_native_plugin_entry_abi_uses_typed_error() {
    let native_plugin_abi =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_abi.rs");
    let native_boundary = include_str!(
        "../../../../../../../../../docs/engine-architecture/native-plugin-boundary.md"
    );
    let review_findings = include_str!(
        "../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let module_convention = include_str!(
        "../../../../../../../../../docs/zircon_runtime/structure/module-convention.md"
    );
    let status_rows = include_str!(
        "../../../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../../../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../../../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "type NativePluginEntryAbiResult<T>",
        "std::result::Result<T, NativePluginEntryAbiError>",
        "enum NativePluginEntryAbiError",
        "UnsupportedDescriptorAbiVersion",
        "MissingEntrySymbol",
        "InvalidGrantedCapabilities",
        "NullEntryReport",
        "UnsupportedEntryAbiVersion",
        "InvalidBehavior",
        "InvalidPackageManifest",
        "InvalidBridgeMethods",
        "impl std::fmt::Display for NativePluginEntryAbiError",
        "impl std::error::Error for NativePluginEntryAbiError",
        ") -> NativePluginEntryAbiResult<NativePluginEntryReport>",
        ") -> NativePluginEntryAbiResult<Self>",
        "NativePluginEntryAbiError::MissingEntrySymbol",
        "NativePluginEntryAbiError::InvalidBridgeMethods",
        "NativePluginEntryAbiError::NullEntryReport",
        "native_entry_abi_error_preserves_granted_capability_source",
        "native_entry_abi_error_preserves_unsupported_entry_message",
    ] {
        assert!(
            native_plugin_abi.contains(required),
            "native plugin entry ABI typed-error owner should contain `{required}`"
        );
    }

    let production = native_plugin_abi
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin ABI production source");
    for forbidden in [
        ") -> Result<NativePluginEntryReport, String> {\n    let symbol_name",
        ") -> Result<Self, String> {",
        "return Err(format!(\n            \"unsupported native plugin ABI version",
        ".map_err(|error| format!(\"native plugin entry symbol is missing: {error}\"))?",
        ".map_err(|_| \"native plugin requested capability contained an interior NUL\".to_string())?",
        "return Err(\"native plugin entry returned null\".to_string());",
        "return Err(format!(\n                \"unsupported native plugin entry ABI version",
        "NativePluginBehavior::from_abi_v3(&*abi.behavior)\n                    .map_err(|error| error.to_string())?",
        "package_manifest_from_toml(\n                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),\n                \"native plugin entry package manifest is invalid\",\n            )\n            .map_err(|error| error.to_string())?",
        "bridge_method_bindings_from_abi_v3(abi.bridge_methods)\n                .map_err(|error| error.to_string())?",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin entry ABI owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        native_plugin_abi.contains("call_native_plugin_entry_result(")
            && native_plugin_abi.contains(".map_err(|error| error.to_string())"),
        "native plugin entry call should keep string formatting only at the loader report boundary"
    );

    for doc_anchor in [
        "Runtime 15 F5 native plugin entry ABI typed errors",
        "runtime_15_native_plugin_entry_abi_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_plugin_entry_abi_uses_typed_error",
        "plugin/native_plugin_loader/native_plugin_abi.rs",
        "NativePluginEntryAbiError::MissingEntrySymbol",
        "entry ABI parse keeps string diagnostics at the loader report boundary",
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
            "native plugin entry ABI docs/status should record `{doc_anchor}`"
        );
    }
}
