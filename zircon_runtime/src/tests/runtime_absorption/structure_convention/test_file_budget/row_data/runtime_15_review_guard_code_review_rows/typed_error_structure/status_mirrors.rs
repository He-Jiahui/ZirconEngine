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

pub(super) fn assert_typed_error_structure_assertions_row_data_status_is_current() {
    let review_guard_status_rows =
        read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_ROW_PATH);
    let status_map = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_MAP_PATH);
    let date_map = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_DATE_MAP_PATH);
    let row_data_anchors = [
        "Runtime 15 M3 review-guard typed-error structure-assertions row-data folder-backed split",
        "runtime_15_review_guard_typed_error_structure_assertions_row_data_folder_backed_static_passed_cargo_deferred",
        TYPED_ERROR_STRUCTURE_ASSERTIONS_ROW_DATA_STATUS_PATH,
        "runtime_15_review_guard_typed_error_structure_assertions_row_data_is_folder_backed",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard status-support rows record typed-error structure-assertions row-data split",
        &review_guard_status_rows,
        &row_data_anchors,
    );
    assert_status_docs_include(
        "typed-error structure-assertions row-data split",
        &row_data_anchors,
    );
    assert_contains_all(
        "code-review status/date maps record typed-error structure-assertions row-data split",
        &(status_map + &date_map),
        &[
            "Runtime 15 M3 review-guard typed-error structure-assertions row-data folder-backed split",
            "runtime_15_review_guard_typed_error_structure_assertions_row_data_folder_backed_static_passed_cargo_deferred",
            "2026-07-06",
        ],
    );
}

pub(super) fn assert_typed_error_structure_assertions_guard_status_is_current() {
    let review_guard_status_rows =
        read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_ROW_PATH);
    let status_map = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_MAP_PATH);
    let date_map = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_DATE_MAP_PATH);
    let status_anchors = [
        TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_STATUS_NAME,
        TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_assertions.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/row_routes.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/folder_backed.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure/budgets.rs",
        TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard typed-error status rows record structure-assertions guard split",
        &review_guard_status_rows,
        &status_anchors,
    );
    assert_status_docs_include(
        TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_STATUS_NAME,
        &status_anchors,
    );
    assert_contains_all(
        "code-review status map records typed-error structure-assertions guard split",
        &status_map,
        &[
            TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_STATUS_NAME,
            TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "code-review date map records typed-error structure-assertions guard split",
        &date_map,
        &[
            TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
}
