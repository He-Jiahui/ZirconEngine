use super::*;

#[test]
fn runtime_15_code_review_rows_row_data_owner_is_child_backed() {
    let code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_rows = review_guard_rows_source_blob();
    let structure_guard_rows = structure_guard_rows_source_blob();
    let typed_error_structure_rows = read_runtime_src(TYPED_ERROR_STRUCTURE_ROWS_PATH);
    let typed_error_structure_row_children = typed_error_structure_rows_source_blob();
    let row_data_owner = read_runtime_src(ROW_DATA_OWNER_PATH);

    assert_contains_all(
        "code-review row-data parent mounts child owners",
        &code_review_rows,
        &[
            "#[path = \"code_review_rows/review_guard_rows.rs\"]",
            "#[path = \"code_review_rows/structure_guard_rows.rs\"]",
            "#[path = \"code_review_rows/typed_error_structure_rows.rs\"]",
            "#[path = \"code_review_rows/row_data_owner.rs\"]",
            "review_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "review_guard_rows::P0_EXPECTED_STATUS_OUTPUT_SLICES",
            "review_guard_rows::F8_EXPECTED_STATUS_OUTPUT_SLICES",
            "review_guard_rows::LATE_API_EXPECTED_STATUS_OUTPUT_SLICES",
            "review_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::STATUS_DOC_PATHS_EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::STATUS_DOC_DELEGATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::STATUS_DOC_STATUS_MAPS_EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::STATUS_DOC_STATUS_MIRRORS_EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::STRUCTURE_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !code_review_rows.contains("pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &["),
        "code_review_rows.rs should route child row-data owners instead of owning row tuples directly"
    );

    assert_contains_all(
        "code-review row-data children own representative rows",
        &(review_guard_rows
            + structure_guard_rows.as_str()
            + typed_error_structure_rows.as_str()
            + typed_error_structure_row_children.as_str()
            + row_data_owner.as_str()),
        &[
            "Runtime 15 M3 code review findings test folder split",
            "Runtime 15 M3 late API cleanup review guard child-owner split",
            "Runtime 15 M3 code review findings structure guard child-owner split",
            "Runtime 15 M3 code review findings structure guard typed-error child-owner split",
            STRUCTURE_GUARD_ROW_DATA_STATUS_NAME,
            "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split",
            CODE_REVIEW_ROWS_ROW_DATA_STATUS_NAME,
            STRUCTURE_GUARD_ROW_DATA_STATUS_ID,
            STRUCTURE_GUARD_ROW_DATA_GUARD_NAME,
            CODE_REVIEW_ROWS_ROW_DATA_STATUS_ID,
            CODE_REVIEW_ROWS_ROW_DATA_GUARD_NAME,
            REVIEW_GUARD_ROWS_ROW_DATA_STATUS_ID,
            REVIEW_GUARD_ROWS_ROW_DATA_GUARD_NAME,
            TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_STATUS_ID,
            TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_GUARD_NAME,
        ],
    );
}

fn typed_error_structure_rows_source_blob() -> String {
    [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/core_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_doc_path_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_doc_delegation_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_doc_status_maps_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_doc_status_mirrors_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertion_rows.rs",
        TYPED_ERROR_STRUCTURE_ROW_DATA_OWNER_PATH,
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n")
}
