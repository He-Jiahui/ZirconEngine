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

    for moved_row_doc_source in [
        "lock_poison_status.rs",
        "module_convention_status.rs",
        "review_status_sync.rs",
    ] {
        assert!(
            !status_docs_guard.contains(moved_row_doc_source),
            "runtime_15_m3_child_group_status_docs.rs should delegate row status-doc source {moved_row_doc_source}"
        );
        assert!(
            status_row_docs_guard.contains(moved_row_doc_source),
            "runtime_15_m3_child_group_status_row_docs.rs should own row status-doc source {moved_row_doc_source}"
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
