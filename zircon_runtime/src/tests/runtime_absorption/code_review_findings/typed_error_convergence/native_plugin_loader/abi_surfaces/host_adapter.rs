#[test]
fn review_f5_native_host_api_adapter_uses_typed_error() {
    let host_api_adapter =
        include_str!("../../../../../../plugin/native_plugin_loader/host_api_adapter.rs");
    let host_api_adapter_abi_decode_tests = include_str!(
        "../../../../../../plugin/native_plugin_loader/host_api_adapter/abi_decode/tests.rs"
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
        host_api_adapter.contains("mod abi_decode;"),
        "native host API adapter root should mount the canonical ABI decoding owner"
    );

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
            host_api_adapter_abi_decode_tests.contains(required_test),
            "native host API adapter ABI decode tests should contain `{required_test}`"
        );
    }
}
