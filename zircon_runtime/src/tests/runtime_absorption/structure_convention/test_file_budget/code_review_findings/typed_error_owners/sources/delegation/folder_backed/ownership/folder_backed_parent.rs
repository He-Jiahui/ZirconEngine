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
    let mut budget_sources: Vec<(&'static str, String)> = vec![(
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD,
        folder_backed_parent,
    )];
    budget_sources.extend(typed_error_source_inventory_delegation_folder_backed_child_sources());

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
