use super::*;

#[test]
fn runtime_15_code_review_rows_row_data_owner_is_child_backed() {
    let code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_rows = read_runtime_src(REVIEW_GUARD_ROWS_PATH);
    let structure_guard_rows = structure_guard_rows_source_blob();
    let typed_error_structure_rows = read_runtime_src(TYPED_ERROR_STRUCTURE_ROWS_PATH);
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
            "structure_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "structure_guard_rows::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_structure_rows::EXPECTED_STATUS_OUTPUT_SLICES",
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
        ],
    );
}
