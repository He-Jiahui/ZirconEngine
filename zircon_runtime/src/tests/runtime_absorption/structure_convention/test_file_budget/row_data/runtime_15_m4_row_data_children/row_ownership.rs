use super::*;

#[test]
fn runtime_15_m4_row_data_owner_is_child_backed() {
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let m4 = read_runtime_src(RUNTIME_15_M4_EXPECTED_STATUS_ROW_DATA_PATH);
    let core_rhi_dynamic = read_runtime_src(RUNTIME_15_M4_CORE_RHI_DYNAMIC_ROW_DATA_PATH);
    let asset_scene_render = read_runtime_src(RUNTIME_15_M4_ASSET_SCENE_RENDER_ROW_DATA_PATH);
    let ui_text_template = read_runtime_src(RUNTIME_15_M4_UI_TEXT_TEMPLATE_ROW_DATA_PATH);
    let ui_surface_plugin = read_runtime_src(RUNTIME_15_M4_UI_SURFACE_PLUGIN_ROW_DATA_PATH);
    let row_data_owner = read_runtime_src(RUNTIME_15_M4_ROW_DATA_OWNER_PATH);
    let render_shader_sync = read_runtime_src(RUNTIME_15_M4_RENDER_SHADER_SYNC_ROW_DATA_PATH);

    assert_contains_all(
        "Runtime 15 M4 row-data parent mounts child owners",
        &m4,
        &[
            "#[path = \"m4/core_rhi_dynamic.rs\"]",
            "#[path = \"m4/asset_scene_render.rs\"]",
            "#[path = \"m4/ui_text_template.rs\"]",
            "#[path = \"m4/ui_surface_plugin.rs\"]",
            "#[path = \"m4/row_data_owner.rs\"]",
            "#[path = \"m4/render_shader_sync.rs\"]",
            "core_rhi_dynamic::EXPECTED_STATUS_OUTPUT_SLICES",
            "asset_scene_render::EXPECTED_STATUS_OUTPUT_SLICES",
            "ui_text_template::EXPECTED_STATUS_OUTPUT_SLICES",
            "ui_surface_plugin::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
            "render_shader_sync::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !m4.contains(
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &["
        ),
        "m4.rs should route child row-data owners instead of owning row tuples directly"
    );

    assert_contains_all(
        "Runtime 15 M4 row-data children own representative rows",
        &(core_rhi_dynamic.clone()
            + asset_scene_render.as_str()
            + ui_text_template.as_str()
            + ui_surface_plugin.as_str()
            + row_data_owner.as_str()
            + render_shader_sync.as_str()),
        &[
            "Runtime 15 M4 no oversized production files global gate",
            "Runtime 15 M4 material asset value/readiness helper owner split",
            "Runtime 15 M4 UI text layout engine visual-order owner split",
            "Runtime 15 M4 UI surface event-routing owner split",
            "Runtime 15 M4 shader prewarm owner guard sync",
            ROW_DATA_OWNER_STATUS_NAME,
            ROW_DATA_OWNER_STATUS_ID,
            ROW_DATA_OWNER_GUARD_NAME,
        ],
    );

    assert_contains_all(
        "Runtime 15 row-data parent exports every M4 child group",
        &runtime_15,
        &[
            "RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M4_ASSET_SCENE_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M4_UI_TEXT_TEMPLATE_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M4_UI_SURFACE_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M4_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M4_RENDER_SHADER_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "top-level expected status row data consumes every M4 child group",
        &top_level,
        &[
            "runtime_15::RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_ASSET_SCENE_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_UI_TEXT_TEMPLATE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_UI_SURFACE_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_RENDER_SHADER_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
