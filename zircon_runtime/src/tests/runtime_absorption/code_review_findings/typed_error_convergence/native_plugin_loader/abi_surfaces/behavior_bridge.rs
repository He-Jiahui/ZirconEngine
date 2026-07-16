#[test]
fn review_f5_native_plugin_behavior_abi_uses_typed_error() {
    let behavior_calls =
        include_str!("../../../../../../plugin/native_plugin_loader/behavior_calls.rs");
    let native_plugin_abi =
        include_str!("../../../../../../plugin/native_plugin_loader/native_plugin_abi.rs");
    let native_boundary =
        include_str!("../../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let review_findings =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
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
        "type NativePluginBehaviorResult<T>",
        "std::result::Result<T, NativePluginBehaviorError>",
        "enum NativePluginBehaviorError",
        "UnsupportedAbiVersion",
        "impl std::fmt::Display for NativePluginBehaviorError",
        "impl std::error::Error for NativePluginBehaviorError",
        ") -> NativePluginBehaviorResult<Self>",
        "NativePluginBehaviorError::UnsupportedAbiVersion",
        "native_behavior_reports_unsupported_abi_version_with_typed_error",
        "native_behavior_typed_error_preserves_unsupported_abi_message",
    ] {
        assert!(
            behavior_calls.contains(required),
            "native plugin behavior ABI typed-error owner should contain `{required}`"
        );
    }

    let production = behavior_calls
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin behavior production source");
    for forbidden in [
        "from_abi_v3(abi: &NativePluginBehaviorV3) -> Result<Self, String>",
        "Err(format!(\n                \"unsupported native plugin behavior ABI version",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin behavior owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        native_plugin_abi.contains("NativePluginBehavior::from_abi_v3(&*abi.behavior)")
            && native_plugin_abi
                .contains("NativePluginEntryAbiError::InvalidBehavior { source }"),
        "native plugin ABI entry report should preserve behavior errors inside the entry ABI typed error"
    );

    for doc_anchor in [
        "Runtime 15 F5 native plugin behavior ABI typed errors",
        "runtime_15_native_plugin_behavior_abi_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_plugin_behavior_abi_uses_typed_error",
        "plugin/native_plugin_loader/behavior_calls.rs",
        "NativePluginBehaviorError::UnsupportedAbiVersion",
        "behavior ABI parse is wrapped by the entry ABI typed error before the loader report boundary",
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
            "native plugin behavior ABI docs/status should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_native_bridge_method_abi_uses_typed_error() {
    let bridge_method_abi =
        include_str!("../../../../../../plugin/native_plugin_loader/bridge_method_abi.rs");
    let native_plugin_abi =
        include_str!("../../../../../../plugin/native_plugin_loader/native_plugin_abi.rs");
    let native_boundary =
        include_str!("../../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let review_findings =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
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
        "type NativeBridgeMethodAbiResult<T>",
        "std::result::Result<T, NativeBridgeMethodAbiError>",
        "enum NativeBridgeMethodAbiError",
        "UnsupportedTableAbiVersion",
        "MissingMethodsPointerWithCount",
        "InvalidRequiredField",
        "MissingCallback",
        "impl std::fmt::Display for NativeBridgeMethodAbiError",
        "impl std::error::Error for NativeBridgeMethodAbiError",
        ") -> NativeBridgeMethodAbiResult<Vec<NativeBridgeMethodBinding>>",
        "fn required_bridge_method_field(",
        "NativeBridgeMethodAbiError::UnsupportedTableAbiVersion",
        "NativeBridgeMethodAbiError::MissingCallback",
        "bridge_method_bindings_report_unsupported_table_abi_with_typed_error",
        "bridge_method_typed_error_preserves_missing_callback_message",
    ] {
        assert!(
            bridge_method_abi.contains(required),
            "native bridge method ABI typed-error owner should contain `{required}`"
        );
    }

    let production = bridge_method_abi
        .split("#[cfg(test)]")
        .next()
        .expect("native bridge method ABI production source");
    for forbidden in [
        ") -> Result<Vec<NativeBridgeMethodBinding>, String>",
        "Err(format!(\n            \"unsupported native bridge method table ABI version",
        "native bridge method table declared methods but methods pointer was null\".to_string()",
        "Err(format!(\n                \"native bridge method `{interface_id}.{method_name}` declared no callback\"",
    ] {
        assert!(
            !production.contains(forbidden),
            "native bridge method ABI owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        native_plugin_abi.contains("bridge_method_bindings_from_abi_v3(abi.bridge_methods)")
            && native_plugin_abi
                .contains("NativePluginEntryAbiError::InvalidBridgeMethods { source }"),
        "native plugin ABI entry report should preserve bridge method errors inside the entry ABI typed error"
    );

    for doc_anchor in [
        "Runtime 15 F5 native bridge method ABI typed errors",
        "runtime_15_native_bridge_method_abi_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_bridge_method_abi_uses_typed_error",
        "plugin/native_plugin_loader/bridge_method_abi.rs",
        "NativeBridgeMethodAbiError::UnsupportedTableAbiVersion",
        "bridge method ABI parse is wrapped by the entry ABI typed error before the loader report boundary",
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
            "native bridge method ABI docs/status should record `{doc_anchor}`"
        );
    }
}
