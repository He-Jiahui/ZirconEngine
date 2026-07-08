use super::*;

#[test]
fn runtime_15_status_output_review_guard_row_data_status_docs_are_child_owner() {
    let review_guard_row_data_guard = read_runtime_src(REVIEW_GUARD_ROW_DATA_GUARD_PATH);
    let status_doc_source_children = status_doc_full_source_blob();

    for moved_doc_source in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    ] {
        assert!(
            !review_guard_row_data_guard.contains(moved_doc_source),
            "runtime_15_review_guard_row_data.rs should delegate status-doc source {moved_doc_source}"
        );
        assert!(
            status_doc_source_children.contains(moved_doc_source),
            "runtime_15_review_guard_row_data_status_docs child tree should own status-doc source {moved_doc_source}"
        );
    }
    assert_contains_all(
        "review-guard row-data status-doc guard records this split",
        &status_doc_source_children,
        &[
            STATUS_DOC_CHILD_OWNER_STATUS_NAME,
            STATUS_DOC_CHILD_OWNER_STATUS_ID,
            STATUS_DOC_CHILD_OWNER_GUARD_NAME,
        ],
    );
}
