use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_export_chain_review_guard_splits_are_current() {
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);

    assert_contains_all(
        "review-guard split parent exports direct-assertion code-review group",
        &review_guard_splits,
        &[
            "CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_F12_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_F12_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_F8_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_F8_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_P0_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_P0_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::DIRECT_ASSERTION_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "code_review_rows::PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
