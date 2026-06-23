use super::*;

#[test]
fn runtime_15_asset_facade_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/facade.rs");
    let handle_events = read_runtime_src("asset/tests/facade/handle_events.rs");
    let load_state_roots = read_runtime_src("asset/tests/facade/load_state_roots.rs");
    let project_facade = read_runtime_src("asset/tests/facade/project_facade.rs");
    let recursive_dependencies = read_runtime_src("asset/tests/facade/recursive_dependencies.rs");
    let dependency_failures = read_runtime_src("asset/tests/facade/dependency_failures.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "asset facade parent test module mounts",
        &parent,
        &[
            "mod dependency_failures;",
            "mod failure_reason;",
            "mod handle_events;",
            "mod handle_lifecycle;",
            "mod hot_reload;",
            "mod load_state_roots;",
            "mod project_facade;",
            "mod recursive_dependencies;",
            "fn texture_asset",
            "fn shader_asset",
            "fn ui_v2_view_asset",
        ],
    );

    for moved_test in [
        "fn typed_handle_roundtrips_and_rejects_kind_mismatch",
        "fn asset_load_state_maps_resource_state_runtime_state_and_payload_residency",
        "fn assets_insert_remove_and_project_manager_helpers_use_typed_facade",
        "fn recursive_dependency_load_state_walks_nested_resource_dependencies",
        "fn recursive_dependency_load_state_marks_missing_dependency_as_failed",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/facade.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/facade.rs should not keep executable tests in the parent module"
    );
    let migrated_test_count = [
        handle_events.as_str(),
        load_state_roots.as_str(),
        project_facade.as_str(),
        recursive_dependencies.as_str(),
        dependency_failures.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 20,
        "asset facade child modules should preserve the original 20 parent tests"
    );

    assert_contains_all(
        "asset facade handle/event child owns typed handle and event contracts",
        &handle_events,
        &[
            "use super::*;",
            "fn typed_handle_roundtrips_and_rejects_kind_mismatch",
            "fn assets_get_acquire_release_and_kind_filtering_use_resource_manager_storage",
            "fn typed_asset_events_preserve_rename_reload_and_remove_order",
        ],
    );
    assert_contains_all(
        "asset facade load-state child owns root state contracts",
        &load_state_roots,
        &[
            "use super::*;",
            "fn asset_load_state_maps_resource_state_runtime_state_and_payload_residency",
            "fn load_states_for_missing_wrong_kind_and_non_resident_roots_do_not_restore_payloads",
            "fn readiness_report_marks_missing_and_wrong_kind_roots_without_restoring_payloads",
        ],
    );
    assert_contains_all(
        "asset facade project child owns typed project facade contracts",
        &project_facade,
        &[
            "use super::*;",
            "fn assets_insert_remove_and_project_manager_helpers_use_typed_facade",
            "fn project_asset_manager_load_returns_typed_handle_and_state",
            "fn project_asset_manager_load_accepts_v2_ui_payload_under_ui_layout_kind",
        ],
    );
    assert_contains_all(
        "asset facade recursive dependency child owns nested dependency contracts",
        &recursive_dependencies,
        &[
            "use super::*;",
            "fn recursive_dependency_load_state_walks_nested_resource_dependencies",
            "fn readiness_report_keeps_shallowest_direct_dependency_row_and_terminates_cycles",
            "fn dependency_load_state_reports_first_level_dependency_changes",
        ],
    );
    assert_contains_all(
        "asset facade dependency failure child owns missing/direct precedence contracts",
        &dependency_failures,
        &[
            "use super::*;",
            "fn recursive_dependency_load_state_marks_missing_dependency_as_failed",
            "fn readiness_report_marks_missing_dependency_records_as_failed_rows",
            "fn dependency_load_state_applies_direct_precedence_and_missing_records",
        ],
    );

    for (path, source) in [
        ("asset/tests/facade.rs", parent.as_str()),
        (
            "asset/tests/facade/handle_events.rs",
            handle_events.as_str(),
        ),
        (
            "asset/tests/facade/load_state_roots.rs",
            load_state_roots.as_str(),
        ),
        (
            "asset/tests/facade/project_facade.rs",
            project_facade.as_str(),
        ),
        (
            "asset/tests/facade/recursive_dependencies.rs",
            recursive_dependencies.as_str(),
        ),
        (
            "asset/tests/facade/dependency_failures.rs",
            dependency_failures.as_str(),
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
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset facade test folder split",
                "runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked",
                "asset/tests/facade.rs",
                "asset/tests/facade/recursive_dependencies.rs",
                "runtime_15_asset_facade_tests_are_folder_backed",
            ],
        );
    }
}
