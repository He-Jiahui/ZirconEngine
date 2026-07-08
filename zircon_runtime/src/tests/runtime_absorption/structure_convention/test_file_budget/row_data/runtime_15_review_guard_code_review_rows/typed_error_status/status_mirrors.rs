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

pub(super) fn assert_typed_error_status_doc_row_data_status_is_current() {
    let review_guard_status_rows = read_runtime_src(TYPED_ERROR_STATUS_DOCS_STATUS_ROW_PATH);
    let status_map = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROW_DATA_DATE_MAP_PATH);

    let row_data_anchors = [
        "Runtime 15 M3 review-guard typed-error status-doc row-data folder-backed split",
        "runtime_15_review_guard_typed_error_status_doc_row_data_folder_backed_static_passed_cargo_deferred",
        TYPED_ERROR_STATUS_DOCS_ROW_DATA_STATUS_PATH,
        "runtime_15_review_guard_typed_error_status_docs_row_data_is_folder_backed",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard typed-error status rows record status-doc row-data split",
        &review_guard_status_rows,
        &row_data_anchors,
    );
    assert_status_docs_include("typed-error status-doc row-data split", &row_data_anchors);
    assert_contains_all(
        "status-support status map records typed-error status-doc row-data split",
        &status_map,
        &[
            "Runtime 15 M3 review-guard typed-error status-doc row-data folder-backed split",
            "runtime_15_review_guard_typed_error_status_doc_row_data_folder_backed_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-support date map records typed-error status-doc row-data split",
        &date_map,
        &[
            "Runtime 15 M3 review-guard typed-error status-doc row-data folder-backed split",
            "2026-07-06",
        ],
    );
}

pub(super) fn assert_typed_error_status_doc_guard_status_is_current() {
    let review_guard_status_rows = read_runtime_src(TYPED_ERROR_STATUS_DOCS_STATUS_ROW_PATH);
    let status_map = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROW_DATA_DATE_MAP_PATH);
    let status_anchors = [
        TYPED_ERROR_STATUS_DOCS_GUARD_FOLDER_BACKED_STATUS_NAME,
        TYPED_ERROR_STATUS_DOCS_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status_docs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/row_routes.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/folder_backed.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/budgets.rs",
        TYPED_ERROR_STATUS_DOCS_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard typed-error status rows record status-doc guard split",
        &review_guard_status_rows,
        &status_anchors,
    );
    assert_status_docs_include(
        TYPED_ERROR_STATUS_DOCS_GUARD_FOLDER_BACKED_STATUS_NAME,
        &status_anchors,
    );
    assert_contains_all(
        "status-support status map records typed-error status-doc guard split",
        &status_map,
        &[
            TYPED_ERROR_STATUS_DOCS_GUARD_FOLDER_BACKED_STATUS_NAME,
            TYPED_ERROR_STATUS_DOCS_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "status-support date map records typed-error status-doc guard split",
        &date_map,
        &[
            TYPED_ERROR_STATUS_DOCS_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
}
