use super::*;

#[test]
fn runtime_15_scene_script_row_data_owner_is_child_backed() {
    let scene_script_tests = read_runtime_src(SCENE_SCRIPT_TESTS_ROWS_PATH);
    let runtime_07_performance = read_runtime_src(SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_PATH);
    let runtime_07_primary_guard_rows =
        read_runtime_src(SCENE_SCRIPT_RUNTIME_07_PRIMARY_GUARD_ROWS_PATH);
    let script_vm_runtime = read_runtime_src(SCENE_SCRIPT_SCRIPT_VM_RUNTIME_PATH);
    let plugin_extension_tests = read_runtime_src(SCENE_SCRIPT_PLUGIN_EXTENSION_TESTS_PATH);
    let plugin_extension_export_build_rows =
        read_runtime_src(SCENE_SCRIPT_PLUGIN_EXTENSION_EXPORT_BUILD_ROWS_PATH);
    let script_vm_gameplay_shader = read_runtime_src(SCENE_SCRIPT_GAMEPLAY_SHADER_PATH);
    let scene_ecs_tests = read_runtime_src(SCENE_SCRIPT_SCENE_ECS_TESTS_PATH);
    let scene_asset_world = read_runtime_src(SCENE_SCRIPT_SCENE_ASSET_WORLD_PATH);
    let row_data_owner = read_runtime_src(SCENE_SCRIPT_ROW_DATA_OWNER_PATH);
    let row_children = [
        runtime_07_performance.as_str(),
        runtime_07_primary_guard_rows.as_str(),
        script_vm_runtime.as_str(),
        plugin_extension_tests.as_str(),
        plugin_extension_export_build_rows.as_str(),
        script_vm_gameplay_shader.as_str(),
        scene_ecs_tests.as_str(),
        scene_asset_world.as_str(),
        row_data_owner.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 scene-script row-data parent mounts child owners",
        &scene_script_tests,
        &[
            "#[path = \"scene_script_tests/runtime_07_performance.rs\"]",
            "#[path = \"scene_script_tests/script_vm_runtime.rs\"]",
            "#[path = \"scene_script_tests/plugin_extension_tests.rs\"]",
            "#[path = \"scene_script_tests/script_vm_gameplay_shader.rs\"]",
            "#[path = \"scene_script_tests/scene_ecs_tests.rs\"]",
            "#[path = \"scene_script_tests/scene_asset_world.rs\"]",
            "#[path = \"scene_script_tests/row_data_owner.rs\"]",
            "runtime_07_performance::EXPECTED_STATUS_OUTPUT_SLICES",
            "script_vm_runtime::EXPECTED_STATUS_OUTPUT_SLICES",
            "plugin_extension_tests::EXPECTED_STATUS_OUTPUT_SLICES",
            "script_vm_gameplay_shader::EXPECTED_STATUS_OUTPUT_SLICES",
            "scene_ecs_tests::EXPECTED_STATUS_OUTPUT_SLICES",
            "scene_asset_world::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !scene_script_tests.contains(
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &["
        ),
        "scene_script_tests.rs should route child row-data owners instead of owning row tuples directly"
    );
    assert_contains_all(
        "Runtime 15 scene-script row-data children own representative rows",
        &row_children,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
            "Runtime 15 M3 script VM hot-reload guard child-owner split",
            "Runtime 15 M3 export build plan platform release-adapter test child-owner split",
            "Runtime 15 M3 script VM gameplay host guard child-owner split",
            "Runtime 15 M3 scene ECS schedule conflict graph child folder split",
            "Runtime 15 M3 scene property paths test folder split",
        ],
    );
}
