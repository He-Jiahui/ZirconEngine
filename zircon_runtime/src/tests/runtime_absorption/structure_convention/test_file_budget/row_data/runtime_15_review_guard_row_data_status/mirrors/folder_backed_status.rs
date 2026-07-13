use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_doc_folder_backed_status_is_current() {
    let review_guard_rows = status_support_review_guard_source_blob();

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status_docs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/row_sources.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/status_maps.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/budgets.rs",
        FOLDER_BACKED_GUARD_NAME,
        STATUS_DOC_CHILD_OWNER_GUARD_NAME,
        "Cargo gate deferred",
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
        assert_contains_all(label, &source, &folder_backed_status_anchors);
    }
    assert_contains_all(
        "status-output Runtime 15 M3 production support row data",
        &review_guard_rows,
        &folder_backed_status_anchors,
    );
}
