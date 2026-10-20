//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f5_native_plugin_string_helpers_use_typed_error() {
    let native_strings = include_str!("../../../../plugin/loader/native_strings.rs");
    let native_plugin_abi = include_str!("../../../../plugin/loader/native_plugin_abi.rs");
    let bridge_method_abi = include_str!("../../../../plugin/loader/bridge_method_abi.rs");
    let _native_boundary =
        include_str!("../../../../../../docs/architecture/native-plugin-boundary.md");
    let _review_findings = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let _runtime_15_plan = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md");
    let _runtime_index = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let _convention =
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let _module_convention =
        include_str!("../../../../../../docs/crates/zircon_runtime/structure/module-convention.md");

    let native_strings_tests = include_str!("../../../../plugin/loader/tests/native_strings.rs");
    assert!(
        native_strings
            .replace("\r\n", "\n")
            .contains("#[cfg(test)]\n#[path = \"tests/native_strings.rs\"]\nmod tests;"),
        "exact current module mount in plugin/loader/native_strings.rs"
    );
    for required in [
        "read_required_c_string_reports_missing_field_with_typed_error",
        "native_string_typed_error_preserves_package_manifest_message",
    ] {
        assert!(
            native_strings_tests.contains(required),
            "current plugin/loader/tests/native_strings.rs must retain {required}"
        );
    }

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
    ] {
        assert!(
            native_strings.contains(required),
            "native plugin string helper typed-error owner should contain `{required}`"
        );
    }

    let production = production_section(native_strings);
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
        native_plugin_abi.contains("read_required_descriptor_field(")
            && native_plugin_abi.contains("abi.plugin_id")
            && native_plugin_abi.contains("\"plugin_id\"")
            && native_plugin_abi.contains("PluginLoadError::invalid_payload(")
            && native_plugin_abi.contains("\"package_manifest_toml\"")
            && native_plugin_abi.contains("source"),
        "native plugin ABI should preserve descriptor and entry string helper sources inside the unified loader error"
    );
    assert!(
        bridge_method_abi.contains("read_required_c_string(value, field_name)")
            && bridge_method_abi.contains("source: source.to_string()"),
        "bridge method ABI should keep native string errors inside its typed ABI error"
    );
}
