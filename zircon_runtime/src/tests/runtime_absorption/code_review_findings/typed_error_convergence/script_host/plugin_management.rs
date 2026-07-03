#[test]
fn review_f5_vm_plugin_management_policy_uses_typed_validation_errors() {
    let management_mod = include_str!("../../../../../script/vm/plugin/management_policy/mod.rs");
    let error_owner = include_str!("../../../../../script/vm/plugin/management_policy/error.rs");
    let garbage_collection =
        include_str!("../../../../../script/vm/plugin/management_policy/garbage_collection.rs");
    let memory = include_str!("../../../../../script/vm/plugin/management_policy/memory.rs");
    let policy = include_str!("../../../../../script/vm/plugin/management_policy/policy.rs");
    let script_vm_mod = include_str!("../../../../../script/vm/mod.rs");
    let script_mod = include_str!("../../../../../script/mod.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_13_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_reflection =
        include_str!("../../../../../../../docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let module_doc =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
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
