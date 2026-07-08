use super::*;

#[test]
fn runtime_15_asset_budget_row_data_export_chain_is_current() {
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "Runtime 15 M3 exposes every asset-budget child group",
        &runtime_15_m3,
        &[
            "ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_ASSET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_BUDGET_RENDER_UI_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_NAMING_CORE_ASSET_DYNAMIC_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_NAMING_GRAPHICS_MISC_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 row-data parent exports every asset-budget child group",
        &runtime_15,
        &[
            "RUNTIME_15_M3_ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_ASSET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_BUDGET_RENDER_UI_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_NAMING_CORE_ASSET_DYNAMIC_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_NAMING_GRAPHICS_MISC_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "top-level expected status row data consumes every asset-budget child group",
        &top_level,
        &[
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_ASSET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_BUDGET_RENDER_UI_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_NAMING_CORE_ASSET_DYNAMIC_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_NAMING_GRAPHICS_MISC_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
