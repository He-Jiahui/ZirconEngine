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
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

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
            "fn hot_reload_hooks_can_query_slot_lifecycle_without_deadlocking",
            "fn hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
        ],
    );
    assert_eq!(
        child.matches("#[test]").count(),
        5,
        "hot reload coordinator child should preserve the original 5 module-local tests"
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

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script VM doc", script_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 script VM hot-reload coordinator test folder split",
                "runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred",
                "Runtime 15 M3 script VM hot-reload guard child-owner split",
                "runtime_15_script_vm_hot_reload_guard_child_owner_split_static_passed_cargo_deferred",
                "script/vm/runtime/hot_reload_coordinator.rs",
                "script/vm/runtime/hot_reload_coordinator/tests.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/hot_reload.rs",
                "runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed",
                "runtime_15_script_vm_hot_reload_guard_is_child_owner",
            ],
        );
    }
}
