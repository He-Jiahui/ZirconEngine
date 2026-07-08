use super::*;

#[test]
fn runtime_15_status_output_m3_child_group_status_row_docs_are_child_owner() {
    let status_docs_guard = read_runtime_src(STATUS_DOCS_GUARD_PATH);
    let status_row_docs_guard = format!(
        "{}\n{}\n{}",
        read_runtime_src(STATUS_ROW_DOCS_GUARD_PATH),
        read_runtime_src(ROOT_PATHS_PATH),
        read_runtime_src(ROOT_STATUSES_PATH)
    );

    for (route_parent_source, child_row_source) in [
        ("lock_poison_status.rs", "lock_poison_status/status_rows.rs"),
        (
            "module_convention_status.rs",
            "module_convention_status/status_rows.rs",
        ),
        (
            "review_status_sync.rs",
            "review_status_sync/row_data_owner.rs",
        ),
    ] {
        assert!(
            !status_docs_guard.contains(route_parent_source),
            "runtime_15_m3_child_group_status_docs.rs should delegate row status-doc source {route_parent_source}"
        );
        assert!(
            status_row_docs_guard.contains(child_row_source),
            "runtime_15_m3_child_group_status_row_docs.rs should own row status-doc child source {child_row_source}"
        );
    }
    assert_contains_all(
        "M3 child-group status-row-doc guard records this split",
        &status_row_docs_guard,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
        ],
    );
}
