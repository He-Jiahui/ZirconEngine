use super::*;

#[test]
fn runtime_15_review_guard_row_data_aggregation_exports_are_current() {
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);

    assert_contains_all(
        "top-level status rows include Runtime 15 M3 review-guard row-data groups",
        &parent,
        &[
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !parent.contains("RUNTIME_15_M3_REVIEW_GUARD_SPLITS_EXPECTED_STATUS_OUTPUT_SLICES"),
        "top-level status row groups should consume review-guard topic children directly"
    );
    assert_contains_all(
        "Runtime 15 root delegates M3 review-guard rows",
        &runtime_15,
        &[
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !runtime_15.contains("RUNTIME_15_M3_REVIEW_GUARD_SPLITS_EXPECTED_STATUS_OUTPUT_SLICES"),
        "Runtime 15 status row root should not keep the old monolithic review-guard group"
    );
    assert_contains_all(
        "Runtime 15 M3 root mounts review-guard row-data topic children",
        &runtime_15_m3,
        &[
            "#[path = \"m3/review_guard_splits.rs\"]",
            "mod review_guard_splits;",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !runtime_15_m3.contains("REVIEW_GUARD_SPLITS_EXPECTED_STATUS_OUTPUT_SLICES"),
        "Runtime 15 M3 root should expose review-guard topic groups instead of one monolithic group"
    );
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
            "#[path = \"review_guard_splits/typed_error_rows.rs\"]",
            "mod typed_error_rows;",
            "CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES",
            "TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
