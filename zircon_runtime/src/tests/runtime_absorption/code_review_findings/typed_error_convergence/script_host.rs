#[test]
fn review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary() {
    let gameplay_host = include_str!("../../../../script/vm/gameplay_host.rs");
    let error_owner = include_str!("../../../../script/vm/gameplay_host/error.rs");
    let combat = include_str!("../../../../script/vm/gameplay_host/combat.rs");
    let lifecycle = include_str!("../../../../script/vm/gameplay_host/lifecycle.rs");
    let navigation = include_str!("../../../../script/vm/gameplay_host/navigation.rs");
    let transform = include_str!("../../../../script/vm/gameplay_host/transform.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_doc =
        include_str!("../../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    assert!(
        gameplay_host.contains("mod error;"),
        "gameplay host root should mount the typed error owner"
    );
    for required in [
        "pub(super) type GameplayHostResult<T> = std::result::Result<T, GameplayHostError>;",
        "pub(super) enum GameplayHostError",
        "Scene(#[from] SceneError)",
        "Navigation(#[from] NavigationError)",
        "Json(#[from] serde_json::Error)",
        "MissingEntity",
        "impl From<GameplayHostError> for ScriptHostError",
    ] {
        assert!(
            error_owner.contains(required),
            "gameplay host typed-error owner should contain `{required}`"
        );
    }

    for (label, source) in [
        ("combat", combat),
        ("lifecycle", lifecycle),
        ("navigation", navigation),
        ("transform", transform),
    ] {
        assert!(
            source.contains("GameplayHostResult"),
            "{label} should use GameplayHostResult for internal fallible work"
        );
        for forbidden in [
            "Result<u64, String>",
            "Result<bool, String>",
            "Result<DamageReport, String>",
            ".map_err(|error| error.to_string())",
            "Err(format!(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String-error branch `{forbidden}`"
            );
        }
    }

    for doc_anchor in [
        "Runtime 15 F5 gameplay host typed errors",
        "runtime_15_gameplay_host_typed_errors_static_passed_cargo_deferred",
        "review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary",
        "script/vm/gameplay_host/error.rs",
        "GameplayHostError",
        "GameplayHostResult",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || host_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F5 gameplay host docs/status should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_script_scene_hook_uses_typed_errors_before_core_boundary() {
    let scene_hook = include_str!("../../../../script/vm/scene_hook.rs");
    let error_owner = include_str!("../../../../script/vm/scene_hook/error.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_reflection =
        include_str!("../../../../../../docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    assert!(
        scene_hook.contains("mod error;"),
        "script scene hook root should mount the typed error owner"
    );
    assert!(
        scene_hook.contains("ScriptSceneHookResult"),
        "script scene hook should use ScriptSceneHookResult for internal fallible work"
    );
    assert!(
        scene_hook.contains("ScriptSceneHookError"),
        "script scene hook should use ScriptSceneHookError before the CoreError boundary"
    );
    for required in [
        "pub(super) type ScriptSceneHookResult<T> = std::result::Result<T, ScriptSceneHookError>;",
        "pub(super) enum ScriptSceneHookError",
        "Core(#[from] CoreError)",
        "InvalidBindingComponent",
        "ExportCall",
        "source: serde_json::Error",
        "source: VmError",
    ] {
        assert!(
            error_owner.contains(required),
            "script scene hook typed-error owner should contain `{required}`"
        );
    }
    for required in [
        "CoreError::Initialization",
        "\"ScriptSceneRuntimeHook\".to_string()",
        "error.to_string()",
    ] {
        assert!(
            scene_hook.contains(required),
            "script scene hook should stringify only at the CoreError boundary: `{required}`"
        );
    }
    for forbidden in [
        "Result<(), String>",
        "Result<Vec<EntityScriptBindings>, String>",
        ".map_err(|error| error.to_string())",
        "Err(format!(",
        "format!(\"script binding",
        "format!(\"invalid {SCRIPT_BINDINGS_COMPONENT}",
    ] {
        assert!(
            !scene_hook.contains(forbidden),
            "script scene hook should not keep lossy String-error branch `{forbidden}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 script scene hook typed errors",
        "runtime_15_script_scene_hook_typed_errors_static_passed_cargo_deferred",
        "review_f5_script_scene_hook_uses_typed_errors_before_core_boundary",
        "script/vm/scene_hook/error.rs",
        "ScriptSceneHookError",
        "ScriptSceneHookResult",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || host_reflection.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F5 script scene hook docs/status should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_vm_plugin_management_policy_uses_typed_validation_errors() {
    let management_mod = include_str!("../../../../script/vm/plugin/management_policy/mod.rs");
    let error_owner = include_str!("../../../../script/vm/plugin/management_policy/error.rs");
    let garbage_collection =
        include_str!("../../../../script/vm/plugin/management_policy/garbage_collection.rs");
    let memory = include_str!("../../../../script/vm/plugin/management_policy/memory.rs");
    let policy = include_str!("../../../../script/vm/plugin/management_policy/policy.rs");
    let script_vm_mod = include_str!("../../../../script/vm/mod.rs");
    let script_mod = include_str!("../../../../script/mod.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_13_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_reflection =
        include_str!("../../../../../../docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "mod error;",
        "pub use error::{VmPluginManagementPolicyError, VmPluginManagementPolicyResult};",
    ] {
        assert!(
            management_mod.contains(required),
            "VM plugin management policy module should mount and export typed errors with `{required}`"
        );
    }
    for required in [
        "pub type VmPluginManagementPolicyResult<T>",
        "pub enum VmPluginManagementPolicyError",
        "GarbageCollectionDisabledWithInterval",
        "GarbageCollectionIntervalFramesZero",
        "MemorySoftLimitBytesZero",
        "MemoryHardLimitBytesZero",
        "MemorySoftLimitExceedsHardLimit",
    ] {
        assert!(
            error_owner.contains(required),
            "VM plugin management policy typed-error owner should contain `{required}`"
        );
    }
    for required in [
        "VmPluginManagementPolicyError,",
        "VmPluginManagementPolicyResult,",
    ] {
        assert!(
            script_vm_mod.contains(required) && script_mod.contains(required),
            "script facade exports should include `{required}`"
        );
    }

    let management_production = management_mod
        .split("#[cfg(test)]")
        .next()
        .expect("management policy production module");
    for (label, source) in [
        ("management module", management_production),
        ("garbage collection policy", garbage_collection),
        ("memory policy", memory),
        ("management policy", policy),
    ] {
        assert!(
            source.contains("VmPluginManagementPolicyResult"),
            "{label} should return VmPluginManagementPolicyResult"
        );
        for forbidden in ["Result<(), String>", "Err(format!(", ".to_string())"] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String validation branch `{forbidden}`"
            );
        }
    }

    for doc_anchor in [
        "Runtime 15 F5 VM plugin management policy typed errors",
        "runtime_15_vm_plugin_management_policy_typed_errors_static_passed_cargo_deferred",
        "review_f5_vm_plugin_management_policy_uses_typed_validation_errors",
        "script/vm/plugin/management_policy/error.rs",
        "VmPluginManagementPolicyError",
        "VmPluginManagementPolicyResult",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_13_plan.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || host_reflection.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F5 VM plugin management policy docs/status should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary() {
    let root = include_str!("../../../../bin/zircon_host_reflection_docs.rs");
    let args = include_str!("../../../../bin/zircon_host_reflection_docs/args.rs");
    let error_owner = include_str!("../../../../bin/zircon_host_reflection_docs/error.rs");
    let run = include_str!("../../../../bin/zircon_host_reflection_docs/run.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_reflection =
        include_str!("../../../../../../docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let function_ledger =
        include_str!("../../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in ["mod args;", "mod error;", "mod run;", "run::run("] {
        assert!(
            root.contains(required),
            "host reflection docs bin root should stay a thin entry shell with `{required}`"
        );
    }

    for required in [
        "pub struct HostReflectionDocsArgs",
        "pub fn parse(",
        ") -> HostReflectionDocsResult<HostReflectionDocsArgs>",
        "HostReflectionDocsError::Usage",
        "pub type HostReflectionDocsResult<T> = std::result::Result<T, HostReflectionDocsError>;",
        "pub enum HostReflectionDocsError",
        "CollectBuiltInHostModules",
        "source: VmError",
        "WriteHostInterfaceDocs",
        "source: io::Error",
        "pub fn run(args: impl IntoIterator<Item = OsString>) -> HostReflectionDocsResult<()>",
        "HostReflectionDocsError::CollectBuiltInHostModules",
        "HostReflectionDocsError::WriteHostInterfaceDocs",
    ] {
        assert!(
            args.contains(required) || error_owner.contains(required) || run.contains(required),
            "host reflection docs typed-error path should contain `{required}`"
        );
    }

    for (label, source) in [("root", root), ("args", args), ("run", run)] {
        for forbidden in [
            "fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String>",
            "Result<(), String>",
            "Err(format!(",
            ".map_err(|error| format!(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String-error branch `{forbidden}`"
            );
        }
    }

    for doc_anchor in [
        "Runtime 15 F5 host reflection docs CLI typed errors",
        "runtime_15_host_reflection_docs_cli_typed_errors_static_passed_cargo_deferred",
        "review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary",
        "bin/zircon_host_reflection_docs/error.rs",
        "HostReflectionDocsError::CollectBuiltInHostModules",
        "HostReflectionDocsError::WriteHostInterfaceDocs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || host_reflection.contains(doc_anchor)
                || function_ledger.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F5 host reflection docs CLI docs/status should record `{doc_anchor}`"
        );
    }
}
