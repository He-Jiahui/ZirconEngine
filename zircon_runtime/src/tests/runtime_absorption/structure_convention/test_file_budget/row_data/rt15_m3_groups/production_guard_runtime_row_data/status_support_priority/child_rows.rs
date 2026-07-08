use super::*;

const STATUS_SUPPORT_PRIORITY_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "expected_slice_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_EXPECTED_SLICE_ROWS_PATH,
        "Runtime 15 M3 status-support expected-slice row-data owner folder-backed split",
    ),
    (
        "row_data_guard_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_GUARD_ROWS_PATH,
        "Runtime 15 M3 status-support row-data guard folder-backed split",
    ),
    (
        "priority_plan_docs_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_PRIORITY_PLAN_DOCS_ROWS_PATH,
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split",
    ),
    (
        "row_data_owner",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_OWNER_ROWS_PATH,
        PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_CHILD_SPLIT_STATUS_NAME,
    ),
];

pub(super) fn assert_status_support_priority_child_rows_are_route_owned() {
    let route = read_runtime_src(
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROWS_PATH,
    );

    assert_contains_all(
        "status-support priority route mounts child row groups",
        &route,
        &[
            "status_support_priority_rows/expected_slice_rows.rs",
            "status_support_priority_rows/row_data_guard_rows.rs",
            "status_support_priority_rows/priority_plan_docs_rows.rs",
            "status_support_priority_rows/row_data_owner.rs",
            "expected_slice_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "priority_plan_docs_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 status-support expected-slice row-data owner folder-backed split",
        "Runtime 15 M3 status-support row-data guard folder-backed split",
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split",
    ] {
        assert!(
            !route.contains(moved_row),
            "status_support_priority_rows.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in STATUS_SUPPORT_PRIORITY_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "status-support priority child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            route.contains(&format!("mod {module_name};")),
            "status_support_priority_rows.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 90,
            "{path} should stay below its focused status-support priority row-data budget"
        );
    }
}
