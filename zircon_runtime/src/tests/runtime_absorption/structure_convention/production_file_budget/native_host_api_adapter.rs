use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_native_host_api_adapter_tests_are_child_owner() {
    let parent = read_runtime_src("plugin/native_plugin_loader/host_api_adapter.rs");
    let abi_decode_tests =
        read_runtime_src("plugin/native_plugin_loader/host_api_adapter/abi_decode/tests.rs");
    let bridge_scope =
        read_runtime_src("plugin/native_plugin_loader/host_api_adapter/bridge_scope/mod.rs");
    let bridge_scope_tests =
        read_runtime_src("plugin/native_plugin_loader/host_api_adapter/bridge_scope/tests.rs");
    let context_handles_tests =
        read_runtime_src("plugin/native_plugin_loader/host_api_adapter/context_handles/tests.rs");
    let ecs_registration =
        read_runtime_src("plugin/native_plugin_loader/host_api_adapter/ecs_registration/mod.rs");
    let ecs_registration_tests =
        read_runtime_src("plugin/native_plugin_loader/host_api_adapter/ecs_registration/tests.rs");
    let registration_policy_tests = read_runtime_src(
        "plugin/native_plugin_loader/host_api_adapter/registration_policy/tests.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");

    assert_contains_all(
        "native host API adapter root mounts its canonical child owners",
        &parent,
        &[
            "mod abi_decode;",
            "mod bridge_scope;",
            "mod context_handles;",
            "mod ecs_registration;",
            "mod registration_policy;",
        ],
    );
    assert!(
        !parent.contains("mod tests;"),
        "host_api_adapter.rs must not recreate the retired aggregate tests owner"
    );
    assert_contains_all(
        "native host API adapter registration owner keeps ABI entrypoints",
        &ecs_registration,
        &[
            "unsafe extern \"C\" fn native_host_register_system_v1",
            "fn status(code: ZrStatusCode) -> ZrStatus",
        ],
    );
    assert_contains_all(
        "native host API adapter bridge owner keeps bridge entrypoints",
        &bridge_scope,
        &[
            "unsafe extern \"C\" fn native_host_bridge_call_v1",
            "unsafe fn native_host_bridge_call_v1_inner",
        ],
    );
    for moved_test in [
        "fn native_host_api_v3_registers_systems_and_components_into_runtime_registry",
        "fn native_host_bridge_call_scope_dispatches_registered_method",
        "fn native_bridge_method_descriptors_use_package_manifest_metadata",
        "fn native_host_api_v3_preserves_dotted_plugin_ids",
    ] {
        assert!(
            !parent.contains(moved_test),
            "host_api_adapter.rs should delegate {moved_test} to its canonical child owner"
        );
    }
    assert_contains_all(
        "native host API adapter ABI decode owner preserves typed-error coverage",
        &abi_decode_tests,
        &[
            "fn native_host_api_adapter_reports_unknown_stage_with_typed_error",
            "fn native_host_api_adapter_utf8_error_preserves_source",
        ],
    );
    assert_contains_all(
        "native host API adapter registration owner preserves V3 coverage",
        &ecs_registration_tests,
        &[
            "fn native_host_api_v3_registers_systems_and_components_into_runtime_registry",
            "fn native_host_api_v3_preserves_dotted_plugin_ids",
        ],
    );
    assert_contains_all(
        "native host API adapter bridge owner preserves bridge coverage",
        &bridge_scope_tests,
        &[
            "fn native_host_bridge_call_scope_dispatches_registered_method",
            "NativeHostBridgeCallScope::from_method_descriptors",
            "NativeBridgeMethodManifestError::MissingBinding",
            "PluginPackageManifest::new(\"weather\", \"Weather\")",
        ],
    );
    assert_contains_all(
        "native host API adapter context owner preserves generation safety coverage",
        &context_handles_tests,
        &["fn stale_generation_cannot_resolve_reused_slot"],
    );
    assert_contains_all(
        "native host API adapter registration-policy owner preserves V4 coverage",
        &registration_policy_tests,
        &["fn native_host_api_v4_registers_authorized_worker_safe_typed_access"],
    );

    let test_count = [
        abi_decode_tests.as_str(),
        bridge_scope_tests.as_str(),
        context_handles_tests.as_str(),
        ecs_registration_tests.as_str(),
        registration_policy_tests.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert!(
        test_count >= 15,
        "native host API adapter split test owners must retain the established coverage floor; got {test_count}"
    );

    for (path, source) in [
        (
            "plugin/native_plugin_loader/host_api_adapter.rs",
            parent.as_str(),
        ),
        (
            "plugin/native_plugin_loader/host_api_adapter/abi_decode/tests.rs",
            abi_decode_tests.as_str(),
        ),
        (
            "plugin/native_plugin_loader/host_api_adapter/bridge_scope/tests.rs",
            bridge_scope_tests.as_str(),
        ),
        (
            "plugin/native_plugin_loader/host_api_adapter/context_handles/tests.rs",
            context_handles_tests.as_str(),
        ),
        (
            "plugin/native_plugin_loader/host_api_adapter/ecs_registration/tests.rs",
            ecs_registration_tests.as_str(),
        ),
        (
            "plugin/native_plugin_loader/host_api_adapter/registration_policy/tests.rs",
            registration_policy_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production/test owner budget; got {line_count} lines"
        );
    }
}
