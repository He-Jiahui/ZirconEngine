use super::*;

#[test]
fn runtime_15_priority_plan_docs_row_data_export_chain_is_current() {
    let status_support = read_runtime_src(STATUS_SUPPORT_ROW_DATA_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_ROW_DATA_PATH);

    assert_contains_all(
        "status-support row-data parent exports priority-plan-doc child groups",
        &status_support,
        &[
            "PRIORITY_PLAN_DOCS_OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRIORITY_PLAN_DOCS_OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRIORITY_PLAN_DOCS_STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRIORITY_PLAN_DOCS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 row-data parent exports priority-plan-doc child groups",
        &runtime_15_m3,
        &[
            "STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_PRIORITY_PLAN_DOCS_STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_PRIORITY_PLAN_DOCS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 row-data parent exports priority-plan-doc child groups",
        &runtime_15,
        &[
            "RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "top-level expected status row data consumes priority-plan-doc child groups",
        &top_level,
        &[
            "runtime_15::RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
