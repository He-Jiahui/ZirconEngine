use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

pub(super) fn assert_typed_error_native_plugin_loader_routes_are_child_backed() {
    let routes_parent = read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD);
    let route_children = typed_error_native_plugin_loader_route_child_source_blob();

    assert_contains_all(
        "typed-error native plugin loader routes parent mounts focused route children",
        &routes_parent,
        &[
            "#[path = \"routes/abi_surfaces.rs\"]",
            "mod abi_surfaces;",
            "#[path = \"routes/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"routes/lifecycle_paths.rs\"]",
            "mod lifecycle_paths;",
            "#[path = \"routes/live_host.rs\"]",
            "mod live_host;",
            "#[path = \"routes/manifest_sources.rs\"]",
            "mod manifest_sources;",
            "#[path = \"routes/plugin_descriptor.rs\"]",
            "mod plugin_descriptor;",
            "#[path = \"routes/replay_runtime.rs\"]",
            "mod replay_runtime;",
            "#[path = \"routes/status_current.rs\"]",
            "mod status_current;",
            "#[path = \"routes/top_level.rs\"]",
            "mod top_level;",
            "top_level::assert_typed_error_native_plugin_loader_top_level_route_is_folder_backed",
            "abi_surfaces::assert_typed_error_native_abi_surfaces_route_is_folder_backed",
            "plugin_descriptor::assert_typed_error_native_plugin_descriptor_route_is_folder_backed",
            "live_host::assert_typed_error_native_live_host_route_is_folder_backed",
            "lifecycle_paths::assert_typed_error_native_live_host_lifecycle_paths_route_is_folder_backed",
            "replay_runtime::assert_typed_error_native_live_host_replay_runtime_route_is_folder_backed",
            "manifest_sources::assert_typed_error_native_manifest_sources_route_is_folder_backed",
        ],
    );
    for moved_anchor in [
        "#[path = \"native_plugin_loader/abi_surfaces.rs\"]",
        "#[path = \"abi_surfaces/behavior_bridge.rs\"]",
        "#[path = \"plugin_descriptor/string_helpers.rs\"]",
        "#[path = \"live_host/lifecycle_paths.rs\"]",
        "#[path = \"lifecycle_paths/hot_reload.rs\"]",
        "#[path = \"replay_and_runtime/bridge_methods.rs\"]",
        "#[path = \"manifest_sources/compat_registration.rs\"]",
    ] {
        assert!(
            !routes_parent.contains(moved_anchor),
            "native_plugin_loader/routes.rs should delegate `{moved_anchor}` to focused route children"
        );
    }
    for (_, child_path, child_guard) in TYPED_ERROR_NATIVE_STRUCTURE_ROUTE_CHILDREN {
        assert!(
            route_children.contains(child_path),
            "typed-error native plugin loader routes tree should inventory child path {child_path}"
        );
        assert!(
            route_children.contains(child_guard),
            "typed-error native plugin loader routes child should own anchor {child_guard}"
        );
    }
    for (path, source) in [(TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD, routes_parent)]
        .into_iter()
        .chain(typed_error_native_plugin_loader_route_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 native plugin loader route budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_typed_error_native_plugin_loader_routes_are_child_backed() {
    assert_typed_error_native_plugin_loader_routes_are_child_backed();
}
