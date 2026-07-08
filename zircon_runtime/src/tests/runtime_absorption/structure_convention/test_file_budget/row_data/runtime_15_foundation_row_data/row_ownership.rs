use super::*;

#[test]
fn runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let runtime_15_row_data_guard = read_runtime_src(RUNTIME_15_ROW_DATA_GUARD_PATH);
    let guard_parent = read_runtime_src(FOUNDATION_ROW_DATA_GUARD_PATH);
    let guard_sources = format!(
        "{}\n{}\n{}",
        guard_parent,
        read_runtime_src(ROOT_STATUSES_PATH),
        foundation_row_data_child_source_blob()
    );

    assert_contains_all(
        "status-output row-data guard mounts foundation row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/runtime_15_foundation_row_data.rs\"]",
            "mod runtime_15_foundation_row_data;",
        ],
    );
    assert!(
        !runtime_15_row_data_guard.contains(CHILD_OWNER_GUARD_NAME),
        "runtime_15_row_data.rs should delegate foundation row-data checks to its child owner"
    );
    assert_contains_all(
        "foundation row-data guard folder owns the moved guard",
        &guard_sources,
        &[
            CHILD_OWNER_GUARD_NAME,
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
        ],
    );
}
