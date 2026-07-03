use super::*;

#[test]
fn runtime_15_review_guard_moved_row_folder_backed_docs_are_current() {
    let folder_backed_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/delegation.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs",
        "runtime_15_status_output_m3_review_guard_row_data_moved_rows_are_child_owner",
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
            "runtime implementation session",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &folder_backed_anchors);
    }
}
