use super::*;

const NAMING_GRAPHICS_MISC_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "root_route_rows",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_ROOT_ROUTE_ROWS_PATH,
        "Runtime 15 M3 naming-boundary route-owner split",
    ),
    (
        "graphics_asset_rows",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_GRAPHICS_ASSET_ROWS_PATH,
        "Runtime 15 M3 graphics render-framework receiver guard child-owner split",
    ),
    (
        "scene_platform_rows",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_SCENE_PLATFORM_ROWS_PATH,
        "Runtime 15 M3 scene-tests ECS systems guard child-owner split",
    ),
    (
        "plugin_banned_rows",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_PLUGIN_BANNED_ROWS_PATH,
        "Runtime 15 M3 plugin static manifest naming guard child-owner split",
    ),
    (
        "row_data_owner",
        ASSET_BUDGET_NAMING_GRAPHICS_MISC_ROW_DATA_OWNER_PATH,
        NAMING_GRAPHICS_MISC_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
    ),
];

pub(super) fn assert_naming_graphics_misc_child_rows_are_route_owned() {
    let naming_route = read_runtime_src(ASSET_BUDGET_NAMING_GRAPHICS_MISC_PATH);

    assert_contains_all(
        "naming-graphics-misc row-data route mounts child row groups",
        &naming_route,
        &[
            "naming_graphics_misc/root_route_rows.rs",
            "naming_graphics_misc/graphics_asset_rows.rs",
            "naming_graphics_misc/scene_platform_rows.rs",
            "naming_graphics_misc/plugin_banned_rows.rs",
            "naming_graphics_misc/row_data_owner.rs",
            "root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "graphics_asset_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "scene_platform_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "plugin_banned_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 graphics render-framework receiver guard child-owner split",
        "Runtime 15 M3 scene-tests ECS systems guard child-owner split",
        "Runtime 15 M3 plugin static manifest naming guard child-owner split",
    ] {
        assert!(
            !naming_route.contains(moved_row),
            "naming_graphics_misc.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in NAMING_GRAPHICS_MISC_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "naming-graphics-misc child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            naming_route.contains(&format!("mod {module_name};")),
            "naming_graphics_misc.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 130,
            "{path} should stay below its focused row-data budget"
        );
    }
}
