use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_export_chain_is_current() {
    let review_guard_row_data_aggregation =
        read_runtime_src(REVIEW_GUARD_ROW_DATA_AGGREGATION_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let top_level_status_rows = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_rows = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3_rows = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "review-guard row-data folder records direct-assertion export chain",
        &review_guard_row_data_aggregation,
        &[
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "code-review row-data parent mounts direct-assertion child",
        &code_review_rows,
        &[
            "#[path = \"code_review_rows/direct_assertion_rows.rs\"]",
            "mod direct_assertion_rows;",
            "#[path = \"code_review_rows/plugin_importer_rows.rs\"]",
            "mod plugin_importer_rows;",
            "DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "review-guard split parent exports direct-assertion code-review group",
        &review_guard_splits,
        &[
            "CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 status row aggregation consumes direct-assertion group",
        &top_level_status_rows,
        &[
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 root exports direct-assertion review-guard group",
        &runtime_15_rows,
        &[
            "RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 root exports direct-assertion review-guard group",
        &runtime_15_m3_rows,
        &[
            "REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "review_guard_splits::CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "review_guard_splits::CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
