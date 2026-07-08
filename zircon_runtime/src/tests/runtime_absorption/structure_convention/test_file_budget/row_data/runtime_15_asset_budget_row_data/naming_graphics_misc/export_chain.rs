use super::*;

pub(super) fn assert_naming_graphics_misc_exports_are_current() {
    let asset_budget_parent = read_runtime_src(ASSET_BUDGET_ROWS_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "asset-budget parent exports naming-graphics-misc children",
        &asset_budget_parent,
        &[
            "NAMING_GRAPHICS_MISC_GRAPHICS_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
            "NAMING_GRAPHICS_MISC_SCENE_PLATFORM_EXPECTED_STATUS_OUTPUT_SLICES",
            "NAMING_GRAPHICS_MISC_PLUGIN_BANNED_EXPECTED_STATUS_OUTPUT_SLICES",
            "NAMING_GRAPHICS_MISC_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 aggregation exports naming-graphics-misc children",
        &runtime_15_m3,
        &[
            "ASSET_BUDGET_NAMING_GRAPHICS_MISC_GRAPHICS_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_NAMING_GRAPHICS_MISC_SCENE_PLATFORM_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_NAMING_GRAPHICS_MISC_PLUGIN_BANNED_EXPECTED_STATUS_OUTPUT_SLICES",
            "ASSET_BUDGET_NAMING_GRAPHICS_MISC_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 and top-level aggregation consume naming-graphics-misc children",
        &[runtime_15.as_str(), top_level.as_str()].join("\n"),
        &[
            "RUNTIME_15_M3_ASSET_BUDGET_NAMING_GRAPHICS_MISC_GRAPHICS_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_NAMING_GRAPHICS_MISC_SCENE_PLATFORM_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_NAMING_GRAPHICS_MISC_PLUGIN_BANNED_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_ASSET_BUDGET_NAMING_GRAPHICS_MISC_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
