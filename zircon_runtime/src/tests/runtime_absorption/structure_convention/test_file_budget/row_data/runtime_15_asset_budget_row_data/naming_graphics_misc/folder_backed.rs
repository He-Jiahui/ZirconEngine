use super::*;

pub(super) fn assert_naming_graphics_misc_guard_is_folder_backed() {
    let guard_parent = read_runtime_src(ASSET_BUDGET_NAMING_GRAPHICS_MISC_GUARD_PATH);

    assert_contains_all(
        "naming-graphics-misc guard route mounts folder-backed children",
        &guard_parent,
        &[
            "#[path = \"naming_graphics_misc/child_rows.rs\"]",
            "#[path = \"naming_graphics_misc/export_chain.rs\"]",
            "#[path = \"naming_graphics_misc/folder_backed.rs\"]",
            "#[path = \"naming_graphics_misc/status_mirrors.rs\"]",
            "child_rows::assert_naming_graphics_misc_child_rows_are_route_owned();",
            "export_chain::assert_naming_graphics_misc_exports_are_current();",
            "status_mirrors::assert_naming_graphics_misc_row_data_status_is_current();",
            "folder_backed::assert_naming_graphics_misc_guard_is_folder_backed();",
            "status_mirrors::assert_naming_graphics_misc_guard_status_is_current();",
        ],
    );
    for moved_marker in [
        "let naming_route = read_runtime_src",
        "let asset_budget_parent = read_runtime_src",
        "M3 status map records naming-graphics-misc row-data child split",
        "Runtime 15 M3 graphics render-framework receiver guard child-owner split",
    ] {
        assert!(
            !guard_parent.contains(moved_marker),
            "naming_graphics_misc.rs should delegate {moved_marker} to child guard files"
        );
    }
    for (_, path, marker) in NAMING_GRAPHICS_MISC_GUARD_CHILDREN {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "naming-graphics-misc guard child keeps representative assertion",
            &child_source,
            &[*marker],
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused naming-graphics-misc guard budget"
        );
    }
}
