use super::*;

#[test]
fn runtime_15_status_support_row_data_and_budget_children_are_child_owned() {
    let parent = read_runtime_src(STATUS_SUPPORT_ROW_DATA_AND_BUDGET_PATH);
    let status_rows = read_runtime_src(STATUS_SUPPORT_ROW_DATA_RUNTIME_ROW_DATA_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (module_name, path, representative_row) in ROW_DATA_AND_BUDGET_CHILDREN {
        let path_attr = format!("#[path = \"row_data_and_budget/{module_name}.rs\"]");
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "row_data_and_budget parent mounts child row module",
            &parent,
            &[path_attr.as_str(), module_mount.as_str()],
        );

        let child = read_runtime_src(path);
        assert_contains_all(
            path,
            &child,
            &[
                "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
                *representative_row,
            ],
        );
    }

    let status_anchors = [
        ROW_DATA_AND_BUDGET_CHILD_SPLIT_STATUS_NAME,
        ROW_DATA_AND_BUDGET_CHILD_SPLIT_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/runtime_row_data.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/hub_editor_support.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/render_shader_support.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/m3_m4_row_data.rs",
        ROW_DATA_AND_BUDGET_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-support runtime-row rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 M3 status-support map records row-data-and-budget split",
        &status_map,
        &[
            ROW_DATA_AND_BUDGET_CHILD_SPLIT_STATUS_NAME,
            ROW_DATA_AND_BUDGET_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 date map records row-data-and-budget split",
        &date_map,
        &[ROW_DATA_AND_BUDGET_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
