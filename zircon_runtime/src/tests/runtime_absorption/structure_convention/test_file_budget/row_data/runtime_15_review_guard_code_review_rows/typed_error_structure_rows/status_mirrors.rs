use super::*;

const STATUS_MIRROR_DOC_PATHS: &[&str] = &[
    "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/zircon_runtime/structure/module-convention.md",
    ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
];

fn assert_status_docs_include(label: &str, expected: &[&str]) {
    for path in STATUS_MIRROR_DOC_PATHS {
        assert_contains_all(
            &format!("{path} mirrors {label}"),
            &read_repo(path),
            expected,
        );
    }
}

pub(super) fn assert_typed_error_structure_row_data_status_is_current() {
    let row_data_owner = read_runtime_src(TYPED_ERROR_STRUCTURE_ROW_DATA_OWNER_PATH);
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);

    let row_data_anchors = [
        TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_NAME,
        TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/core_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status/paths/core.rs",
        TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all("row-data owner", &row_data_owner, &row_data_anchors);

    let doc_anchors = [
        TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_NAME,
        TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_ID,
        TYPED_ERROR_STRUCTURE_ROWS_PATH,
        TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_status_docs_include(
        TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_NAME,
        &doc_anchors,
    );
    assert_contains_all(
        "review guard status map records typed-error structure row-data split",
        &status_map,
        &[
            TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_NAME,
            TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review guard date map records typed-error structure row-data split",
        &date_map,
        &[
            TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_NAME,
            "2026-07-07",
        ],
    );
}

pub(super) fn assert_typed_error_structure_rows_guard_status_is_current() {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/code_review_rows.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/code_review_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/code_review_maps.rs",
    );
    let status_anchors = [
        TYPED_ERROR_STRUCTURE_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
        TYPED_ERROR_STRUCTURE_ROWS_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/row_groups.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/status_doc_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/folder_backed.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/budgets.rs",
        TYPED_ERROR_STRUCTURE_ROWS_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];

    assert_status_docs_include(
        TYPED_ERROR_STRUCTURE_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
        &status_anchors,
    );
    assert_contains_all(
        "production guard review rows record typed-error structure guard folder-backed split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status map records typed-error structure guard folder-backed split",
        &status_map,
        &[
            TYPED_ERROR_STRUCTURE_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
            TYPED_ERROR_STRUCTURE_ROWS_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records typed-error structure guard folder-backed split",
        &date_map,
        &[
            TYPED_ERROR_STRUCTURE_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
}
