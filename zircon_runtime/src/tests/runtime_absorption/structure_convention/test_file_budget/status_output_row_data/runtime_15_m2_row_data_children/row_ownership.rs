use super::*;

#[test]
fn runtime_15_m2_row_data_owner_is_child_backed() {
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let m2 = read_runtime_src(RUNTIME_15_M2_EXPECTED_STATUS_ROW_DATA_PATH);
    let core_scene_asset_dynamic =
        read_runtime_src(RUNTIME_15_M2_CORE_SCENE_ASSET_DYNAMIC_ROW_DATA_PATH);
    let render_graphics = read_runtime_src(RUNTIME_15_M2_RENDER_GRAPHICS_ROW_DATA_PATH);
    let ui_platform_editor = read_runtime_src(RUNTIME_15_M2_UI_PLATFORM_EDITOR_ROW_DATA_PATH);
    let row_data_owner = read_runtime_src(RUNTIME_15_M2_ROW_DATA_OWNER_PATH);

    assert_contains_all(
        "Runtime 15 M2 row-data parent mounts child owners",
        &m2,
        &[
            "#[path = \"m2/core_scene_asset_dynamic.rs\"]",
            "#[path = \"m2/render_graphics.rs\"]",
            "#[path = \"m2/ui_platform_editor.rs\"]",
            "#[path = \"m2/row_data_owner.rs\"]",
            "core_scene_asset_dynamic::EXPECTED_STATUS_OUTPUT_SLICES",
            "render_graphics::EXPECTED_STATUS_OUTPUT_SLICES",
            "ui_platform_editor::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !m2.contains(
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &["
        ),
        "m2.rs should route child row-data owners instead of owning row tuples directly"
    );

    assert_contains_all(
        "Runtime 15 M2 row-data children own representative rows",
        &(core_scene_asset_dynamic.clone()
            + render_graphics.as_str()
            + ui_platform_editor.as_str()
            + row_data_owner.as_str()),
        &[
            "Runtime 15 M3 status output Runtime 15 M2 row data split",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M2 render shader definition bare-flag naming hard cutover",
            "Runtime 15 M2 editor Workbench archived fixture naming hard cutover",
            ROW_DATA_OWNER_STATUS_NAME,
            ROW_DATA_OWNER_STATUS_ID,
            ROW_DATA_OWNER_GUARD_NAME,
        ],
    );

    assert_contains_all(
        "Runtime 15 row-data parent exports every M2 child group",
        &runtime_15,
        &[
            "RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M2_RENDER_GRAPHICS_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M2_UI_PLATFORM_EDITOR_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M2_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "top-level expected status row data consumes every M2 child group",
        &top_level,
        &[
            "runtime_15::RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M2_RENDER_GRAPHICS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M2_UI_PLATFORM_EDITOR_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M2_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
