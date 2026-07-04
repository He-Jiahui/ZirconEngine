use super::super::super::super::super::super::super::*;
use super::super::super::super::*;

pub(super) fn assert_typed_error_source_inventory_delegation_folder_backed_is_child_owned() {
    let folder_backed_parent =
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD);
    let child_tree = typed_error_source_inventory_delegation_folder_backed_child_source_blob();

    assert_contains_all(
        "typed-error source inventory delegation folder-backed parent mounts focused children",
        &folder_backed_parent,
        &[
            "#[path = \"folder_backed/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"folder_backed/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"folder_backed/status_current.rs\"]",
            "mod status_current;",
            "guard_body::assert_typed_error_source_inventory_guard_is_folder_backed",
            "child_ownership::assert_typed_error_source_inventory_delegation_is_child_backed",
        ],
    );
    for moved_anchor in [
        "typed_error_source_inventory_sources()",
        "typed_error_source_inventory_child_source_blob()",
        "TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_SLICE",
        "TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN",
    ] {
        assert!(
            !folder_backed_parent.contains(moved_anchor),
            "source_inventory/delegation/folder_backed.rs should delegate `{moved_anchor}` to focused children"
        );
    }
    for (_, child_path, child_guard) in
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN
    {
        assert!(
            child_tree.contains(child_path),
            "typed-error source inventory delegation folder-backed tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(child_guard),
            "typed-error source inventory delegation folder-backed child should own anchor {child_guard}"
        );
    }
    for (path, source) in IntoIterator::into_iter([(
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD,
        folder_backed_parent,
    )]
    )
    .chain(typed_error_source_inventory_delegation_folder_backed_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
