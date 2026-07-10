use super::*;

#[test]
fn runtime_15_review_guard_status_support_folder_backed_status_is_current() {
    let review_guard_rows = review_guard_status_support_review_rows_source_blob();
    let status_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH);

    let status_anchors = [
        STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_NAME,
        STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/typed_error_status_doc_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_foundation_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_inventory_metadata_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_delegation_rows.rs",
        STATUS_SUPPORT_ROWS_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard status-support row data records its folder-backed split",
        &review_guard_rows,
        &status_anchors,
    );
    assert_contains_all(
        "review status map records status-support row-data folder-backed split",
        &status_map,
        &[
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_NAME,
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review date map records status-support row-data folder-backed split",
        &date_map,
        &[STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_NAME, "2026-07-05"],
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
