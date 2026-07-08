use super::*;

pub(super) fn assert_status_support_review_rows_row_data_status_is_current() {
    let row_data_owner =
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_OWNER_PATH);
    assert_contains_all(
        "status-support review row-data owner",
        &row_data_owner,
        &[
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_NAME,
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_ID,
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/core_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/status_support_guard_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/typed_error_guard_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows/row_data_guard_rows.rs",
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );

    let doc_anchors = [
        STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_NAME,
        STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_ID,
        REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH,
        STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for path in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
        assert_contains_all(path, &read_repo(path), &doc_anchors);
    }
    assert_contains_all(
        "review guard status map records status-support review row-data split",
        &read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        &[
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_NAME,
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review guard date map records status-support review row-data split",
        &read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        &[
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_NAME,
            "2026-07-07",
        ],
    );
}
