use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_doc_status_mirror_children_are_child_owned() {
    let review_guard_rows = status_support_review_guard_source_blob();
    let status_map = status_support_status_map_source_blob();
    let date_map = status_support_date_map_source_blob();

    let status_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/review_guard_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/status_doc_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_status/mirrors/folder_backed_status.rs",
        "runtime_15_review_guard_row_data_status_doc_status_mirror_children_are_child_owned",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support row data records review-guard status-doc status-mirror child split",
        &review_guard_rows,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support map records review-guard status-doc status-mirror child split",
        &status_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, STATUS_MIRROR_CHILD_SPLIT_ID],
    );
    assert_contains_all(
        "Runtime 15 date map records review-guard status-doc status-mirror child split",
        &date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, "2026-07-04"],
    );

    let doc_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "runtime_15_review_guard_row_data_status_doc_status_mirror_children_are_child_owned",
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
        (
            "session note",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &doc_anchors);
    }
}
