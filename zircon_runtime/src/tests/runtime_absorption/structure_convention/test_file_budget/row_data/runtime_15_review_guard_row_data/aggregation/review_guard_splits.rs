use super::*;

#[test]
fn runtime_15_review_guard_row_data_review_guard_splits_aggregation_is_current() {
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);

    assert_contains_all(
        "review-guard row-data parent routes to topic children",
        &review_guard_splits,
        &[
            "#[path = \"review_guard_splits/code_review_rows.rs\"]",
            "mod code_review_rows;",
            "CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "#[path = \"review_guard_splits/status_support_rows.rs\"]",
            "mod status_support_rows;",
            "STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_REVIEW_GUARD_STATUS_SUPPORT_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_REVIEW_GUARD_TYPED_ERROR_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_TYPED_ERROR_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_SOURCE_INVENTORY_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_SOURCE_INVENTORY_INVENTORY_METADATA_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_SOURCE_INVENTORY_DELEGATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "#[path = \"review_guard_splits/typed_error_rows.rs\"]",
            "mod typed_error_rows;",
            "TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
