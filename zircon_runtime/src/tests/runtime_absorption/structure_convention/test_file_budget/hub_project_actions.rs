use super::{assert_contains_all, read_repo};

const SLICE: &str = "Runtime 15 M3 support Hub project-actions tests child-owner split";
const STATUS: &str =
    "runtime_15_support_hub_project_actions_tests_child_owner_split_static_passed_cargo_deferred";
const DATE: &str = "2026-06-27";
const GUARD: &str = "runtime_15_support_hub_project_actions_tests_are_child_owner";
const PROJECT_ACTIONS: &str = "zircon_hub/src/tauri_app/runtime_state/project_actions.rs";
const PROJECT_ACTION_TESTS: &str =
    "zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs";
const TEST_FILE_BUDGET: usize = 800;

#[test]
fn runtime_15_support_hub_project_actions_tests_are_child_owner() {
    let parent = read_repo(PROJECT_ACTIONS);
    let tests = read_repo(PROJECT_ACTION_TESTS);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let hub_project_doc = read_repo("docs/zircon_hub/projects/lifecycle-workflows.md");
    let status_rows = read_repo(
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/hub_editor_support.rs",
    );
    let status_map = read_repo(
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/hub_editor_maps.rs",
    );
    let date_map = read_repo(
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/hub_editor_maps.rs",
    );

    assert_contains_all(
        "Hub project-actions parent mounts test child owner",
        &parent,
        &["#[path = \"project_actions/tests.rs\"]", "mod tests;"],
    );
    for moved_test in [
        "fn create_project_action_scaffolds_project_and_selects_detail",
        "fn import_project_action_validates_manifest_and_records_recent_project",
        "fn confirm_delete_success_with_injected_recycler_drops_project_only_from_hub",
        "fn session_with_source",
    ] {
        assert!(
            !parent.contains(moved_test),
            "{PROJECT_ACTIONS} should delegate project lifecycle test fixture `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "{PROJECT_ACTION_TESTS} should own project lifecycle test fixture `{moved_test}`"
        );
    }

    for (path, source) in [
        (PROJECT_ACTIONS, parent.as_str()),
        (PROJECT_ACTION_TESTS, tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < TEST_FILE_BUDGET,
            "{path} should stay below {TEST_FILE_BUDGET} lines after the Hub project-action test split; got {line_count}"
        );
    }

    assert_contains_all(
        "Hub project lifecycle docs mention split owner",
        &hub_project_doc,
        &[
            PROJECT_ACTIONS,
            PROJECT_ACTION_TESTS,
            "project-actions tests child-owner split",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("Hub project lifecycle doc", hub_project_doc.as_str()),
        ("status row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[SLICE, STATUS, PROJECT_ACTIONS, PROJECT_ACTION_TESTS, GUARD],
        );
    }
    assert_contains_all("status map", &status_map, &[SLICE, STATUS]);
    assert_contains_all("date map", &date_map, &[SLICE, DATE]);
}
