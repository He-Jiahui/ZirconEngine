use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_doc_status_doc_status_is_current() {
    let review_guard_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);

    assert_contains_all(
        "Runtime 15 M3 production-support row data records review-guard status-doc split",
        &review_guard_rows,
        &[
            REVIEW_GUARD_CHILD_OWNER_STATUS_NAME,
            REVIEW_GUARD_CHILD_OWNER_STATUS_ID,
            STATUS_DOC_CHILD_OWNER_STATUS_NAME,
            STATUS_DOC_CHILD_OWNER_STATUS_ID,
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_status_docs.rs",
            STATUS_DOC_CHILD_OWNER_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );

    let child_owner_status_anchors = [
        STATUS_DOC_CHILD_OWNER_STATUS_NAME,
        STATUS_DOC_CHILD_OWNER_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_status_docs.rs",
        STATUS_DOC_CHILD_OWNER_GUARD_NAME,
    ];
    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &child_owner_status_anchors);
    }
}
