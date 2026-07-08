use super::*;

const PLUGIN_EXTENSION_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "native_loader_rows",
        SCENE_SCRIPT_PLUGIN_EXTENSION_NATIVE_LOADER_ROWS_PATH,
        "Runtime 15 M3 native live-host tests folder split",
    ),
    (
        "manifest_package_rows",
        SCENE_SCRIPT_PLUGIN_EXTENSION_MANIFEST_PACKAGE_ROWS_PATH,
        "Runtime 15 M3 manifest contributions runtime-family test child-owner split",
    ),
    (
        "runtime_catalog_rows",
        SCENE_SCRIPT_PLUGIN_EXTENSION_RUNTIME_CATALOG_ROWS_PATH,
        "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split",
    ),
    (
        "export_build_rows",
        SCENE_SCRIPT_PLUGIN_EXTENSION_EXPORT_BUILD_ROWS_PATH,
        "Runtime 15 M3 export build plan platform release-adapter test child-owner split",
    ),
    (
        "row_data_owner",
        SCENE_SCRIPT_PLUGIN_EXTENSION_ROW_DATA_OWNER_PATH,
        PLUGIN_EXTENSION_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
    ),
];

pub(super) fn assert_plugin_extension_child_rows_are_route_owned() {
    let plugin_route = read_runtime_src(SCENE_SCRIPT_PLUGIN_EXTENSION_TESTS_PATH);

    assert_contains_all(
        "plugin-extension row-data route mounts child row groups",
        &plugin_route,
        &[
            "plugin_extension_tests/native_loader_rows.rs",
            "plugin_extension_tests/manifest_package_rows.rs",
            "plugin_extension_tests/runtime_catalog_rows.rs",
            "plugin_extension_tests/export_build_rows.rs",
            "plugin_extension_tests/row_data_owner.rs",
            "native_loader_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "manifest_package_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_catalog_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "export_build_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 extension registry bridge test folder split",
        "Runtime 15 M3 manifest contributions runtime-family test child-owner split",
        "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split",
        "Runtime 15 M3 export build plan platform release-adapter test child-owner split",
    ] {
        assert!(
            !plugin_route.contains(moved_row),
            "plugin_extension_tests.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in PLUGIN_EXTENSION_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "plugin-extension child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            plugin_route.contains(&format!("mod {module_name};")),
            "plugin_extension_tests.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused row-data budget"
        );
    }
}
