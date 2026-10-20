//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_f5_native_host_api_adapter_uses_typed_error() {
    let host_api_adapter = include_str!("../../../../../plugin/loader/host_api_adapter.rs");
    let errors = include_str!("../../../../../plugin/loader/host_api_adapter/abi_decode/error.rs");
    let decode_read =
        include_str!("../../../../../plugin/loader/host_api_adapter/abi_decode/read.rs");
    let decode_system =
        include_str!("../../../../../plugin/loader/host_api_adapter/abi_decode/system.rs");
    let ecs_registration =
        include_str!("../../../../../plugin/loader/host_api_adapter/ecs_registration/mod.rs");
    let registration_scope =
        include_str!("../../../../../plugin/loader/host_api_adapter/registration_policy/scope.rs");
    let abi_decode =
        include_str!("../../../../../plugin/loader/host_api_adapter/abi_decode/mod.rs");
    let registration_policy =
        include_str!("../../../../../plugin/loader/host_api_adapter/registration_policy/mod.rs");
    for mount in [
        "#[path = \"host_api_adapter/abi_decode/mod.rs\"]",
        "#[path = \"host_api_adapter/ecs_registration/mod.rs\"]",
        "#[path = \"host_api_adapter/registration_policy/mod.rs\"]",
    ] {
        assert!(host_api_adapter.contains(mount), "{mount}");
    }
    for mount in ["error.rs", "read.rs", "system.rs", "tests/cases.rs"] {
        assert!(
            abi_decode.contains(&format!("#[path = \"{mount}\"]")),
            "{mount}"
        );
    }
    assert!(registration_policy.contains("#[path = \"scope.rs\"]"));
    let host_api_adapter_abi_decode_tests =
        include_str!("../../../../../plugin/loader/host_api_adapter/abi_decode/tests/cases.rs");

    for required in [
        "type NativeHostApiAdapterResult<T>",
        "Result<T, NativeHostApiAdapterError>",
        "enum NativeHostApiAdapterError",
        "InvalidPluginModuleOwner",
        "AbiDecode {",
        "impl From<AbiDecodeError> for NativeHostApiAdapterError",
        "InvalidSystemSet",
        "RegisterSystem",
        "UnknownPluginModuleOwner",
        "RegisterComponent",
        "impl std::fmt::Display for NativeHostApiAdapterError",
        "impl std::error::Error for NativeHostApiAdapterError",
    ] {
        assert!(
            errors.contains(required),
            "native host API adapter typed-error owner should contain `{required}`"
        );
    }

    assert!(
        decode_read.contains("unsafe fn read_utf8(slice: ZrByteSlice) -> AbiDecodeResult<String>")
    );
    assert!(decode_read.contains("AbiDecodeError::InvalidUtf8"));
    assert!(decode_system.contains("fn stage_from_abi(stage: u32) -> AbiDecodeResult<SystemStage>"));
    assert!(decode_system.contains(") -> AbiDecodeResult<Vec<String>>"));
    assert!(decode_system.contains("AbiDecodeError::UnknownSystemStage"));
    assert!(registration_scope.contains(") -> NativeHostApiAdapterResult<Self>"));
    assert!(ecs_registration.contains(") -> NativeHostApiAdapterResult<()>"));
    for variant in ["RegisterSystem", "RegisterComponent"] {
        assert!(ecs_registration.contains(&format!("NativeHostApiAdapterError::{variant}")));
    }
    for production in [
        errors,
        decode_read,
        decode_system,
        registration_scope,
        ecs_registration,
    ] {
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
    }

    assert!(
        host_api_adapter.contains("mod abi_decode;"),
        "native host API adapter root should mount the canonical ABI decoding owner"
    );

    assert!(
        registration_scope.contains("Self::new_result(registry, module_name, policy).map_err(|error| error.to_string())")
            && ecs_registration.contains("Err(_) => status(ZrStatusCode::Error),"),
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
