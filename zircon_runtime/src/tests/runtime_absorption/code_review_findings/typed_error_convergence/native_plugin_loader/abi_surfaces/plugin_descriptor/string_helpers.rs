#[test]
fn review_f5_native_plugin_string_helpers_use_typed_error() {
    let native_strings =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_strings.rs");
    let native_plugin_abi =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_abi.rs");
    let bridge_method_abi =
        include_str!("../../../../../../../plugin/native_plugin_loader/bridge_method_abi.rs");
    let native_boundary = include_str!(
        "../../../../../../../../../docs/engine-architecture/native-plugin-boundary.md"
    );
    let review_findings = include_str!(
        "../../../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../../../docs/plans/engine-code-structure-convention.md");
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
        "type NativeStringResult<T>",
        "std::result::Result<T, NativeStringError>",
        "enum NativeStringError",
        "MissingRequiredField",
        "InvalidPackageManifest",
        "impl std::fmt::Display for NativeStringError",
        "impl std::error::Error for NativeStringError",
        ") -> NativeStringResult<String>",
        ") -> NativeStringResult<Option<PluginPackageManifest>>",
        "NativeStringError::MissingRequiredField",
        "NativeStringError::InvalidPackageManifest",
        "read_required_c_string_reports_missing_field_with_typed_error",
        "native_string_typed_error_preserves_package_manifest_message",
    ] {
        assert!(
            native_strings.contains(required),
            "native plugin string helper typed-error owner should contain `{required}`"
        );
    }

    let production = native_strings
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin string helper production source");
    for forbidden in [
        ") -> Result<String, String>",
        ") -> Result<Option<PluginPackageManifest>, String>",
        "ok_or_else(|| format!(\"native plugin descriptor field",
        ".map_err(|error| format!(\"{invalid_message}: {error}\"))",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin string helper owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        native_plugin_abi.contains("read_required_descriptor_field(abi.plugin_id, \"plugin_id\")")
            && native_plugin_abi
                .contains("NativePluginDescriptorAbiError::InvalidRequiredField")
            && native_plugin_abi.contains("NativePluginDescriptorAbiError::InvalidPackageManifest")
            && native_plugin_abi.contains("NativePluginEntryAbiError::InvalidPackageManifest"),
        "native plugin ABI should keep descriptor string helpers typed and wrap entry string helpers inside the entry ABI typed error"
    );
    assert!(
        bridge_method_abi.contains("read_required_c_string(value, field_name)")
            && bridge_method_abi.contains("source: source.to_string()"),
        "bridge method ABI should keep native string errors inside its typed ABI error"
    );

    for doc_anchor in [
        "Runtime 15 F5 native plugin string helper typed errors",
        "runtime_15_native_plugin_string_helper_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_plugin_string_helpers_use_typed_error",
        "plugin/native_plugin_loader/native_strings.rs",
        "NativeStringError::InvalidPackageManifest",
        "native string helpers keep string diagnostics at descriptor and entry loader-report boundaries",
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
            "native plugin string helper docs/status should record `{doc_anchor}`"
        );
    }
}
