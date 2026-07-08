use super::*;

pub(super) fn assert_asset_tests_exports_are_current() {
    let asset_budget_parent = read_runtime_src(ASSET_BUDGET_ROWS_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "asset-budget parent exports asset-tests children",
        &asset_budget_parent,
        &[
            "ASSET_TESTS_ASSET_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_TESTS_UI_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_TESTS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 aggregation exports asset-tests children",
        &runtime_15_m3,
        &[
            "ASSET_BUDGET_ASSET_TESTS_ASSET_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_ASSET_TESTS_UI_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_ASSET_TESTS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 and top-level aggregation consume asset-tests children",
        &[runtime_15.as_str(), top_level.as_str()].join("\n"),
        &[
            "RUNTIME_15_M3_ASSET_BUDGET_ASSET_TESTS_ASSET_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_ASSET_TESTS_UI_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_ASSET_TESTS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
