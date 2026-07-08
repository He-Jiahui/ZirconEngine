use super::*;

pub(super) fn assert_runtime_07_performance_exports_are_current() {
    let scene_script_parent = read_runtime_src(SCENE_SCRIPT_TESTS_ROWS_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "scene-script parent exports Runtime 07 performance children",
        &scene_script_parent,
        &[
            "RUNTIME_07_PERFORMANCE_SPLIT_LAYOUT_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_07_PERFORMANCE_OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_07_PERFORMANCE_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 aggregation exports Runtime 07 performance children",
        &runtime_15_m3,
        &[
            "SCENE_SCRIPT_TESTS_RUNTIME_07_PERFORMANCE_SPLIT_LAYOUT_EXPECTED_STATUS_OUTPUT_SLICES",
            "SCENE_SCRIPT_TESTS_RUNTIME_07_PERFORMANCE_OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
            "SCENE_SCRIPT_TESTS_RUNTIME_07_PERFORMANCE_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 and top-level aggregation consume Runtime 07 performance children",
        &[runtime_15.as_str(), top_level.as_str()].join("\n"),
        &[
            "RUNTIME_15_M3_SCENE_SCRIPT_TESTS_RUNTIME_07_PERFORMANCE_SPLIT_LAYOUT_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_SCENE_SCRIPT_TESTS_RUNTIME_07_PERFORMANCE_OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_SCENE_SCRIPT_TESTS_RUNTIME_07_PERFORMANCE_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
