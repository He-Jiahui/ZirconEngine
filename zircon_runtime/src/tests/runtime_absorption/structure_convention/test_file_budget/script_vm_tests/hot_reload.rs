use super::super::*;

pub(super) fn assert_hot_reload_coordinator_tests_are_folder_backed() {
    let parent = read_runtime_src("script/vm/runtime/hot_reload_coordinator.rs");
    let child = read_runtime_src("script/vm/runtime/hot_reload_coordinator/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let script_doc = read_repo("docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");

    assert_contains_all(
        "hot reload coordinator parent mounts test child owner",
        &parent,
        &["#[cfg(test)]", "mod tests;"],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "hot_reload_coordinator.rs should mount a child test owner instead of keeping executable tests"
    );

    for moved_test in [
        "fn hot_reload_policy_preserves_state_and_increments_generation_by_default",
        "fn stateless_hot_reload_policy_skips_state_transfer",
        "fn disabled_hot_reload_policy_rejects_reload_without_deactivating_slot",
        "fn failed_hot_reload_load_discards_generation_registrations_before_retry",
        "fn hot_reload_hooks_can_query_slot_lifecycle_without_deadlocking",
        "fn hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
    ] {
        assert!(
            !parent.contains(moved_test),
            "hot_reload_coordinator.rs should mount child test owner instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "hot reload coordinator child owns lifecycle, policy, and poison tests",
        &child,
        &[
            "use super::*;",
            "struct PolicyRecordingBackend",
            "struct LifecycleQueryBackend",
            "struct CoordinatorSlotLifecycle",
            "fn hot_reload_policy_preserves_state_and_increments_generation_by_default",
            "fn failed_hot_reload_load_discards_generation_registrations_before_retry",
            "fn hot_reload_hooks_can_query_slot_lifecycle_without_deadlocking",
            "fn hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
        ],
    );
    assert_eq!(
        child.matches("#[test]").count(),
        6,
        "hot reload coordinator child should preserve the 5 split-time tests plus the failed-generation registration retry regression"
    );

    for (path, source) in [
        (
            "script/vm/runtime/hot_reload_coordinator.rs",
            parent.as_str(),
        ),
        (
            "script/vm/runtime/hot_reload_coordinator/tests.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
