use super::{assert_contains_all, read_repo};

const SLICE: &str =
    "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split";
const STATUS: &str =
    "runtime_15_editor_retained_host_workbench_window_projection_tests_child_owner_split_static_passed_cargo_deferred";
const DATE: &str = "2026-06-27";
const GUARD: &str =
    "runtime_15_editor_retained_host_workbench_window_projection_tests_are_child_owner";
const WORKBENCH_PROJECTION: &str =
    "zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs";
const WORKBENCH_PROJECTION_TESTS: &str =
    "zircon_editor/src/ui/retained_host/ui/workbench_window_projection/tests.rs";
const PARENT_FILE_BUDGET: usize = 1000;
const TEST_FILE_BUDGET: usize = 800;

#[test]
fn runtime_15_editor_retained_host_workbench_window_projection_tests_are_child_owner() {
    let parent = read_repo(WORKBENCH_PROJECTION);
    let tests = read_repo(WORKBENCH_PROJECTION_TESTS);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let editor_workbench_doc = read_repo("docs/editor-and-tooling/editor-workbench-shell.md");
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
        "editor retained-host workbench projection parent mounts test child owner",
        &parent,
        &[
            "#[path = \"workbench_window_projection/tests.rs\"]",
            "mod tests;",
        ],
    );
    for moved_test in [
        "fn workbench_button_text_prefers_authored_label_over_value_render_text",
        "fn workbench_input_text_keeps_rendered_value_display_semantics",
        "fn workbench_segmented_control_projects_selected_value_text",
        "fn test_host_node",
    ] {
        assert!(
            !parent.contains(moved_test),
            "{WORKBENCH_PROJECTION} should delegate retained-host test fixture `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "{WORKBENCH_PROJECTION_TESTS} should own retained-host test fixture `{moved_test}`"
        );
    }

    assert!(
        parent.lines().count() < PARENT_FILE_BUDGET,
        "{WORKBENCH_PROJECTION} should stay below the {PARENT_FILE_BUDGET}-line large-file hotspot threshold after the split"
    );
    assert!(
        tests.lines().count() < TEST_FILE_BUDGET,
        "{WORKBENCH_PROJECTION_TESTS} should stay below {TEST_FILE_BUDGET} lines after the split"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("editor workbench doc", editor_workbench_doc.as_str()),
        ("status row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                WORKBENCH_PROJECTION,
                WORKBENCH_PROJECTION_TESTS,
                GUARD,
            ],
        );
    }
    assert_contains_all("status map", &status_map, &[SLICE, STATUS]);
    assert_contains_all("date map", &date_map, &[SLICE, DATE]);
}
