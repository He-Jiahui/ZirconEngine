use super::*;

pub(super) fn assert_status_support_priority_exports_are_current() {
    let runtime_row_data = read_runtime_src(PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ROWS_PATH);
    let production_guard_support = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "production guard runtime row-data exports status-support priority child groups",
        &[runtime_row_data.as_str(), production_guard_support.as_str()].join("\n"),
        &[
            "STATUS_SUPPORT_PRIORITY_ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_PRIORITY_PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_PRIORITY_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 and top-level aggregation consume status-support priority child groups",
        &[
            runtime_15_m3.as_str(),
            runtime_15.as_str(),
            top_level.as_str(),
        ]
        .join("\n"),
        &[
            "PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
