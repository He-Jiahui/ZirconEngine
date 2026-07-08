use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_native_plugin_loader_routes_source_helpers_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD);
    let child_tree = typed_error_native_plugin_loader_route_source_helper_child_source_blob();

    assert_contains_all(
        "typed-error native plugin loader routes parent mounts source helper children",
        &parent,
        &[
            "#[path = \"routes/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"routes/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"routes/sources.rs\"]",
            "mod sources;",
            "#[path = \"routes/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"routes/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "pub(super) use child_inventory::*;",
            "pub(super) use metadata::*;",
            "pub(super) use sources::*;",
            "assert_typed_error_native_plugin_loader_routes_are_folder_backed",
        ],
    );
    for moved_anchor in [
        "pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD_OWNERSHIP_CHILD",
        "pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_ROUTE_CHILDREN",
        "pub(in super::super) fn typed_error_native_plugin_loader_route_child_sources",
        "pub(in super::super) fn typed_error_native_plugin_loader_route_child_source_blob",
        "TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SPLIT",
        "TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_STATUS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error native plugin loader route source helper `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_SOURCE_HELPER_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error native plugin loader route source helper tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error native plugin loader route source helper child {child_path} should own anchor {anchor}"
        );
    }
    for (path, source) in [(TYPED_ERROR_NATIVE_STRUCTURE_ROUTES_CHILD, parent)]
        .into_iter()
        .chain(typed_error_native_plugin_loader_route_source_helper_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 native plugin loader route source helper budget; got {line_count} lines"
        );
    }
}
