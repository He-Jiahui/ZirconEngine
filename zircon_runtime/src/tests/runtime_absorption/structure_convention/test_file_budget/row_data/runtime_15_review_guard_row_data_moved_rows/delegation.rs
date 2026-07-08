use super::*;

#[test]
fn runtime_15_status_output_m3_review_guard_row_data_moved_rows_are_child_owner() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let review_guard_row_data_guard = read_runtime_src(REVIEW_GUARD_ROW_DATA_PATH);
    let moved_rows_guard = read_runtime_src(MOVED_ROWS_PARENT_PATH);
    let child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = moved_row_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts review-guard moved-row child",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/runtime_15_review_guard_row_data_moved_rows.rs\"]",
            "mod runtime_15_review_guard_row_data_moved_rows;",
        ],
    );
    assert!(
        !review_guard_row_data_guard.contains(
            "fn runtime_15_status_output_m3_review_guard_row_data_moved_rows_are_child_owner"
        ),
        "runtime_15_review_guard_row_data.rs should delegate moved-row checks"
    );
    assert_contains_all(
        "review-guard moved-row parent mounts folder-backed children",
        &moved_rows_guard,
        &[
            "mod code_review_rows;",
            "mod delegation;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_paths;",
            "mod root_source_blobs;",
            "mod root_statuses;",
            "mod status_mirrors;",
            "mod typed_error_rows;",
        ],
    );
    assert_contains_all(
        "review-guard moved-row parent records old and new split anchors",
        &status_inventory,
        MOVED_ROWS_STATUS_ANCHORS,
    );

    for (_, child_path, guard_name) in MOVED_ROWS_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "review-guard moved-row child inventory should list child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "review-guard moved-row child {child_path} should define {guard_name}"
        );
    }
}
