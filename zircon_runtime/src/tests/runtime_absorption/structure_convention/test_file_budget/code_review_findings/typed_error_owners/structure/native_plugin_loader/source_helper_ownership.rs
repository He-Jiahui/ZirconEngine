use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_native_plugin_loader_source_helpers_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD);
    let child_tree = typed_error_native_plugin_loader_source_helper_child_source_blob();

    assert_contains_all(
        "typed-error native plugin loader parent mounts source helper children",
        &parent,
        &[
            "#[path = \"native_plugin_loader/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"native_plugin_loader/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"native_plugin_loader/sources.rs\"]",
            "mod sources;",
            "#[path = \"native_plugin_loader/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"native_plugin_loader/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "pub(super) use child_inventory::*;",
            "pub(super) use metadata::*;",
            "pub(super) use sources::*;",
            "runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
        ],
    );
    for moved_anchor in [
        "pub(super) struct TypedErrorNativePluginLoaderSources",
        "pub(super) fn typed_error_native_plugin_loader_sources",
        "pub(super) fn typed_error_native_plugin_loader_child_sources",
        "pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_GUARD_CHILDREN",
        "const REVIEW_GUARD_STATUS_ROWS_PATH",
        "TYPED_ERROR_NATIVE_STRUCTURE_BUDGETS_CHILD",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error native plugin loader source helper `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_NATIVE_STRUCTURE_SOURCE_HELPER_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error native plugin loader source helper tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error native plugin loader source helper child {child_path} should own anchor {anchor}"
        );
    }
    for (path, source) in [(TYPED_ERROR_NATIVE_STRUCTURE_CHILD, parent)]
        .into_iter()
        .chain(typed_error_native_plugin_loader_source_helper_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
