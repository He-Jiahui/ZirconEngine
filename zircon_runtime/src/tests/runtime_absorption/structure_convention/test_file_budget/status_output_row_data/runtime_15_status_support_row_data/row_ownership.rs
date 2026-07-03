use super::*;

#[test]
fn runtime_15_status_support_row_data_owner_is_child_backed() {
    let status_support = read_runtime_src(STATUS_SUPPORT_ROWS_PATH);
    let row_data_and_budget = read_runtime_src(STATUS_SUPPORT_ROW_DATA_AND_BUDGET_PATH);
    let expected_slice_maps = read_runtime_src(STATUS_SUPPORT_EXPECTED_SLICE_MAPS_PATH);
    let runtime_index_anchors = read_runtime_src(STATUS_SUPPORT_RUNTIME_INDEX_ANCHORS_PATH);
    let priority_plan_docs = read_runtime_src(STATUS_SUPPORT_PRIORITY_PLAN_DOCS_PATH);
    let priority_plan_docs_integrity = read_runtime_src(PRIORITY_PLAN_DOCS_INTEGRITY_PATH);
    let priority_plan_docs_owner = read_runtime_src(PRIORITY_PLAN_DOCS_OWNER_PATH);
    let priority_plan_docs_followups = read_runtime_src(PRIORITY_PLAN_DOCS_FOLLOWUPS_PATH);
    let priority_plan_docs_row_data = read_runtime_src(PRIORITY_PLAN_DOCS_ROW_DATA_PATH);
    let row_children = [
        row_data_and_budget.as_str(),
        expected_slice_maps.as_str(),
        runtime_index_anchors.as_str(),
        priority_plan_docs.as_str(),
        priority_plan_docs_integrity.as_str(),
        priority_plan_docs_owner.as_str(),
        priority_plan_docs_followups.as_str(),
        priority_plan_docs_row_data.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 status-support row-data parent mounts child owners",
        &status_support,
        &[
            "#[path = \"status_support/row_data_and_budget.rs\"]",
            "#[path = \"status_support/expected_slice_maps.rs\"]",
            "#[path = \"status_support/runtime_index_anchors.rs\"]",
            "#[path = \"status_support/priority_plan_docs.rs\"]",
            "row_data_and_budget::EXPECTED_STATUS_OUTPUT_SLICES",
            "expected_slice_maps::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_index_anchors::EXPECTED_STATUS_OUTPUT_SLICES",
            "priority_plan_docs::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !status_support.contains("pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &["),
        "status_support.rs should route child row-data owners instead of owning row tuples directly"
    );
    assert_contains_all(
        "Runtime 15 status-support row-data children own representative rows",
        &row_children,
        &[
            "Runtime 15 M3 test file budget root-layout child split",
            "Runtime 15 M3 status output expected-slice maps split",
            "Runtime 15 M3 runtime index subplan map 01-15 sync",
            "Runtime 15 M3 priority plan docs code-path integrity guard",
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
        ],
    );
}
