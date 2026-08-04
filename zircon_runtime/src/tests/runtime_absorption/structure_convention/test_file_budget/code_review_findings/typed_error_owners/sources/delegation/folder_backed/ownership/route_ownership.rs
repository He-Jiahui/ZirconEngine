use super::super::super::super::super::super::super::*;
use super::super::super::super::*;

pub(super) fn assert_typed_error_source_inventory_delegation_folder_backed_ownership_is_child_backed(
) {
    let ownership_parent = read_runtime_src(
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD_OWNERSHIP_CHILD,
    );
    let ownership_children =
        typed_error_source_inventory_delegation_folder_backed_ownership_child_source_blob();

    assert_contains_all(
        "typed-error source inventory delegation folder-backed ownership parent mounts focused children",
        &ownership_parent,
        &[
            "#[path = \"ownership/delegation_parent.rs\"]",
            "mod delegation_parent;",
            "#[path = \"ownership/folder_backed_parent.rs\"]",
            "mod folder_backed_parent;",
            "#[path = \"ownership/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"ownership/route_ownership.rs\"]",
            "mod route_ownership;",
            "#[path = \"ownership/status_current.rs\"]",
            "mod status_current;",
            "delegation_parent::assert_typed_error_source_inventory_delegation_is_child_backed",
            "folder_backed_parent::assert_typed_error_source_inventory_delegation_folder_backed_is_child_owned",
            "route_ownership::assert_typed_error_source_inventory_delegation_folder_backed_ownership_is_child_backed",
            "budgets::assert_typed_error_source_inventory_delegation_folder_backed_ownership_child_budgets_are_current",
        ],
    );
    for moved_anchor in [
        "typed_error_source_inventory_delegation_child_source_blob()",
        "typed_error_source_inventory_delegation_folder_backed_child_source_blob()",
        "sources/delegation.rs should delegate",
        "sources/delegation/folder_backed.rs should delegate",
    ] {
        assert!(
            !ownership_parent.contains(moved_anchor),
            "sources/delegation/folder_backed/child_ownership.rs should delegate `{moved_anchor}` to focused children"
        );
    }
    for (_, child_path, child_guard) in
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN
    {
        assert!(
            ownership_children.contains(child_path),
            "typed-error source inventory delegation folder-backed ownership tree should inventory child path {child_path}"
        );
        assert!(
            ownership_children.contains(child_guard),
            "typed-error source inventory delegation folder-backed ownership child should own anchor {child_guard}"
        );
    }
}
