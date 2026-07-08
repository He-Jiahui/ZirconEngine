use super::super::super::super::super::super::*;
use super::super::super::*;
use super::*;

pub(in super::super) fn assert_typed_error_source_inventory_child_inventory_status_current_is_child_backed(
) {
    let child_inventory_parent =
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_CHILD);
    let child_inventory_children = typed_error_source_inventory_child_inventory_child_source_blob();
    let status_current_parent =
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CHILD);
    let status_current_children = status_current_child_source_blob();

    assert_contains_all(
        "typed-error source inventory child inventory parent mounts focused children",
        &child_inventory_parent,
        &[
            "#[path = \"inventory/delegation_children.rs\"]",
            "#[path = \"inventory/folder_backed_children.rs\"]",
            "#[path = \"inventory/folder_backed_ownership_children.rs\"]",
            "#[path = \"inventory/root_children.rs\"]",
            "#[path = \"inventory/source_helper_children.rs\"]",
            "#[path = \"inventory/status_current.rs\"]",
            "pub(super) use delegation_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN;",
            "pub(super) use folder_backed_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN;",
            "pub(super) use folder_backed_ownership_children::TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN;",
            "pub(super) use root_children::TYPED_ERROR_SOURCE_INVENTORY_CHILDREN;",
            "pub(super) use source_helper_children::TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN;",
        ],
    );
    for moved_anchor in [
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_CHILDREN: &[",
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_SOURCE_HELPER_CHILDREN: &[",
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN: &[",
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN: &[",
        "pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN: &[",
    ] {
        assert!(
            !child_inventory_parent.contains(moved_anchor),
            "sources/child_inventory.rs should delegate `{moved_anchor}` to focused children"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_CHILDREN {
        let child_mount = child_path
            .strip_prefix("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/")
            .unwrap_or(child_path);
        assert!(
            child_inventory_parent.contains(child_mount),
            "typed-error source inventory child inventory parent should mount child path {child_mount}"
        );
        assert!(
            child_inventory_children.contains(anchor),
            "typed-error source inventory child inventory child {child_path} should own anchor {anchor}"
        );
    }
    assert_contains_all(
        "typed-error source inventory child inventory status-current parent mounts focused children",
        &status_current_parent,
        &[
            "#[path = \"current/budgets.rs\"]",
            "#[path = \"current/route_ownership.rs\"]",
            "#[path = \"current/status_mirrors.rs\"]",
            "route_ownership::assert_typed_error_source_inventory_child_inventory_status_current_is_child_backed",
            "status_mirrors::assert_typed_error_source_inventory_child_inventory_status_current_mirrors_are_current",
            "budgets::assert_typed_error_source_inventory_child_inventory_status_current_child_budgets_are_current",
        ],
    );
    for (_, child_path, anchor) in
        TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_CHILDREN
    {
        let child_mount = child_path
            .strip_prefix("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/inventory/")
            .unwrap_or(child_path);
        assert!(
            status_current_parent.contains(child_mount),
            "typed-error source inventory child inventory status-current parent should mount child path {child_mount}"
        );
        assert!(
            status_current_children.contains(anchor),
            "typed-error source inventory child inventory status-current child {child_path} should own anchor {anchor}"
        );
    }
}

fn status_current_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in TYPED_ERROR_SOURCE_INVENTORY_CHILD_INVENTORY_STATUS_CURRENT_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
