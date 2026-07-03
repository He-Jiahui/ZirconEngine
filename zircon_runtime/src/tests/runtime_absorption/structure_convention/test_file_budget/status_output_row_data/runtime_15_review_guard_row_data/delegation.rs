use super::*;

#[test]
fn runtime_15_status_output_m3_review_guard_row_data_is_child_owner() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let runtime_15_row_data_guard = read_runtime_src(RUNTIME_15_ROW_DATA_GUARD_PATH);
    let review_guard_row_data_guard = read_runtime_src(REVIEW_GUARD_ROW_DATA_GUARD_PATH);
    let child_sources = review_guard_row_data_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts review-guard row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_review_guard_row_data.rs\"]",
            "mod runtime_15_review_guard_row_data;",
            "#[path = \"status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs\"]",
            "mod runtime_15_review_guard_row_data_moved_rows;",
        ],
    );
    assert!(
        !runtime_15_row_data_guard.contains(CHILD_OWNER_GUARD_NAME),
        "runtime_15_row_data.rs should delegate review-guard row-data checks to its child owner"
    );
    assert_contains_all(
        "review-guard row-data parent mounts folder-backed children",
        &review_guard_row_data_guard,
        &[
            "mod aggregation;",
            "mod budgets;",
            "mod delegation;",
            "mod moved_rows;",
            "mod status_mirrors;",
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in REVIEW_GUARD_ROW_DATA_CHILDREN {
        assert!(
            review_guard_row_data_guard.contains(child_path),
            "review-guard row-data parent should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "review-guard row-data child {child_path} should define {guard_name}"
        );
    }
}
