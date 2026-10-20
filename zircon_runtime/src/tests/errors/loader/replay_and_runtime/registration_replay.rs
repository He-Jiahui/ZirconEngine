//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f5_native_live_host_registration_replay_uses_typed_error() {
    let registration_replay = include_str!("../../../../plugin/loader/live/registration_replay.rs");
    let registration_replay_error =
        include_str!("../../../../plugin/loader/live/registration_replay/error.rs");
    let live_host_root = include_str!("../../../../plugin/loader/live.rs");
    let registration_replay_tests =
        include_str!("../../../../plugin/loader/live/tests/registration_replay.rs");
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

    for required in [
        "type NativePluginRegistrationReplayResult<T>",
        "std::result::Result<T, NativePluginRegistrationReplayError>",
        ") -> NativePluginRegistrationReplayResult<NativePluginRuntimeRegistrationReplayReport>",
        ") -> NativePluginRegistrationReplayResult<RuntimeRegistrationManifestSource>",
        ") -> NativePluginRegistrationReplayResult<NativePluginRuntimeRegistrationSystemReplay>",
        ") -> NativePluginRegistrationReplayResult<()>",
        "NativePluginRegistrationReplayError::UnsupportedManifestSchema",
        "NativePluginRegistrationReplayError::InvalidRegistrationManifest",
        "NativePluginRegistrationReplayError::RegisterNativeSystem",
    ] {
        assert!(
            registration_replay.contains(required),
            "native live-host registration replay orchestration owner should contain `{required}`"
        );
    }

    for required in [
        "enum NativePluginRegistrationReplayError",
        "UnsupportedManifestSchema",
        "MissingRegistrationManifest",
        "InvalidRegistrationManifest",
        "InvalidRegistrationSystem",
        "UnknownBridgeInterface",
        "RegistryInternPluginModule",
        "RegistryInternSystemSet",
        "RegisterNativeSystem",
        "impl std::fmt::Display for NativePluginRegistrationReplayError",
        "impl std::error::Error for NativePluginRegistrationReplayError",
    ] {
        assert!(
            registration_replay_error.contains(required),
            "native live-host registration replay error owner should contain `{required}`"
        );
    }

    let production = [
        production_section(registration_replay),
        production_section(registration_replay_error),
    ]
    .concat();
    for forbidden in [
        "return Err(format!(\n                    \"runtime plugin {plugin_id} registration manifest schema",
        ".map_err(|error| format!(\"runtime plugin {plugin_id} {error}\"))",
        "format!(\n                    \"runtime plugin {plugin_id} registration system `{}` references unknown bridge interface",
        ".map_err(|error| error.to_string())?",
        "format!(\n            \"runtime plugin {plugin_id} failed to register native registration manifest system",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host registration replay owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        registration_replay.contains(".map_err(|error| error.to_string())")
            && registration_replay.contains("report.diagnostics.push(error);"),
        "public replay report boundaries may keep string diagnostics after typed replay errors are displayed"
    );
    assert!(
        live_host_root.contains("NativePluginRegistrationReplayError"),
        "native live-host test root should expose the typed registration replay error to module-local tests"
    );

    let registration_replay_structure_tests =
        include_str!("../../../../plugin/loader/live/tests/registration_replay/structure.rs");
    assert!(
        registration_replay_tests
            .replace("\r\n", "\n")
            .contains("#[path = \"registration_replay/structure.rs\"]\nmod structure;"),
        "exact current module mount in plugin/loader/live/tests/registration_replay.rs"
    );
    for required_test in [
        "native_registration_replay_reports_typed_schema_error",
        "native_registration_replay_reports_typed_duplicate_system_error",
        "NativePluginRegistrationReplayError::UnsupportedManifestSchema",
        "NativePluginRegistrationReplayError::RegisterNativeSystem",
    ] {
        assert!(
            registration_replay_structure_tests.contains(required_test),
            "native live-host registration replay tests should contain `{required_test}`"
        );
    }
}
