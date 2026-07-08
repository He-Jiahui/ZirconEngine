use super::*;

#[test]
fn runtime_15_review_guard_row_data_runtime_15_root_aggregation_is_current() {
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);

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
}
