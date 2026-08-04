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
}
