#[test]
fn review_f5_native_plugin_descriptor_abi_uses_typed_error() {
    let native_plugin_abi =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_abi.rs");
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
        "type NativePluginDescriptorAbiResult<T>",
        "std::result::Result<T, NativePluginDescriptorAbiError>",
        "enum NativePluginDescriptorAbiError",
        "NullDescriptorSymbol",
        "UnsupportedAbiVersion",
        "InvalidRequiredField",
        "InvalidPackageManifest",
        "impl std::fmt::Display for NativePluginDescriptorAbiError",
        "impl std::error::Error for NativePluginDescriptorAbiError",
        "NativePluginDescriptorAbiError::NullDescriptorSymbol",
        "NativePluginDescriptorAbiError::UnsupportedAbiVersion",
        "NativePluginDescriptorAbiError::InvalidPackageManifest",
        "unsafe fn from_abi_v3(abi: &NativePluginAbiV3) -> NativePluginDescriptorAbiResult<Self>",
        "fn read_required_descriptor_field(",
    ] {
        assert!(
            native_plugin_abi.contains(required),
            "native plugin descriptor ABI typed-error owner should contain `{required}`"
        );
    }

    let production = native_plugin_abi
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin ABI production source");
    for forbidden in [
        "return Err(\"native plugin ABI v3 descriptor symbol returned null\".to_string());",
        "unsafe fn from_abi_v3(abi: &NativePluginAbiV3) -> Result<Self, String>",
        "return Err(format!(\n                \"unsupported native plugin ABI version",
        "read_required_c_string(abi.plugin_id, \"plugin_id\")\n            .map_err(|error| error.to_string())?",
        ".map_err(|error| error.to_string())?,\n            runtime_entry_name",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin descriptor ABI owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        native_plugin_abi.contains("NativePluginDescriptor::from_abi_v3(&*descriptor)")
            && native_plugin_abi.contains(".map_err(|error| error.to_string())"),
        "native plugin descriptor probe should keep string formatting at the loader report boundary"
    );

    for doc_anchor in [
        "Runtime 15 F5 native plugin descriptor ABI typed errors",
        "runtime_15_native_plugin_descriptor_abi_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_plugin_descriptor_abi_uses_typed_error",
        "plugin/native_plugin_loader/native_plugin_abi.rs",
        "NativePluginDescriptorAbiError::UnsupportedAbiVersion",
        "descriptor ABI parse keeps string diagnostics at the loader report boundary",
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
            "native plugin descriptor ABI docs/status should record `{doc_anchor}`"
        );
    }
}
