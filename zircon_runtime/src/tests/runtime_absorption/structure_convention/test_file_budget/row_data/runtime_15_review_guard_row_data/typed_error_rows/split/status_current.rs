use super::*;

#[test]
fn runtime_15_review_guard_typed_error_rows_guard_status_is_current() {
    let status_rows = read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH);
    let status_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH);

    let status_anchors = [
        TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
        TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/typed_error_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/typed_error_rows/route_children.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/typed_error_rows/representative_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/typed_error_rows/export_chain.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/typed_error_rows/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/typed_error_rows/split_layout.rs",
        TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard status-support rows record typed-error rows guard folder-backed split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "review status map records typed-error rows guard folder-backed split",
        &status_map,
        &[
            TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
            TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review date map records typed-error rows guard folder-backed split",
        &date_map,
        &[
            TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-06",
        ],
    );

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
            "Frameworks 02 plan",
            "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
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
        assert_contains_all(label, &source, &status_anchors);
    }
}
