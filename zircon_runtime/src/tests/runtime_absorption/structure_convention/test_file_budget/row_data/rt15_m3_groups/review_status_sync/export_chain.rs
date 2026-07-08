use super::super::*;

pub(super) fn assert_review_status_sync_export_chain_is_current() {
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "Runtime 15 M3 aggregation exports review status-sync children",
        &runtime_15_m3,
        &[
            "REVIEW_STATUS_SYNC_P0_CORE_EXPECTED_STATUS_OUTPUT_SLICES",
            "REVIEW_STATUS_SYNC_TYPED_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES",
            "REVIEW_STATUS_SYNC_PROVIDER_LOOKUP_EXPECTED_STATUS_OUTPUT_SLICES",
            "REVIEW_STATUS_SYNC_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 and top-level aggregation consume review status-sync children",
        &[runtime_15.as_str(), top_level.as_str()].join("\n"),
        &[
            "RUNTIME_15_M3_REVIEW_STATUS_SYNC_P0_CORE_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_REVIEW_STATUS_SYNC_TYPED_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_REVIEW_STATUS_SYNC_PROVIDER_LOOKUP_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_REVIEW_STATUS_SYNC_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
