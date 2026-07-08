use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_source_inventory_source_helpers_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD);
    let child_tree = typed_error_source_inventory_source_helper_child_source_blob();

    assert_contains_all(
        "typed-error source inventory parent mounts source helper children",
        &parent,
        &[
            "#[path = \"sources/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"sources/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"sources/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"sources/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"sources/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "pub(super) use child_sources::*;",
            "pub(super) use child_inventory::*;",
            "pub(super) use metadata::*;",
            "runtime_15_typed_error_source_inventory_is_child_owner",
        ],
    );
    for moved_anchor in [
        "pub(super) struct TypedErrorSourceInventorySources",
        "pub(super) fn typed_error_source_inventory_sources",
        "pub(super) fn typed_error_source_inventory_child_sources",
        "pub(super) fn typed_error_source_inventory_child_source_blob",
        "TYPED_ERROR_SOURCE_INVENTORY_CHILDREN",
        "REVIEW_GUARD_STATUS_ROWS_PATH",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error source inventory source helper `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error source inventory source helper tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error source inventory source helper child {child_path} should own anchor {anchor}"
        );
    }
    let mut budget_sources: Vec<(&'static str, String)> =
        vec![(TYPED_ERROR_SOURCE_INVENTORY_CHILD, parent)];
    budget_sources.extend(typed_error_source_inventory_source_helper_child_sources());

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
