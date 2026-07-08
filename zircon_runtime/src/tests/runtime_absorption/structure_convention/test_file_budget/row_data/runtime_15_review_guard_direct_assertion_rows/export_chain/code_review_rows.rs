use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_export_chain_code_review_rows_are_current() {
    let code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);

    assert_contains_all(
        "code-review row-data parent mounts direct-assertion child",
        &code_review_rows,
        &[
            "#[path = \"code_review_rows/direct_assertion_rows.rs\"]",
            "mod direct_assertion_rows;",
            "#[path = \"code_review_rows/plugin_importer_rows.rs\"]",
            "mod plugin_importer_rows;",
            "DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_F12_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_F8_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_P0_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
