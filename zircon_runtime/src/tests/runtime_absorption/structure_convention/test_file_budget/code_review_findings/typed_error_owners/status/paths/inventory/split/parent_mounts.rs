use super::super::super::super::super::super::super::*;
use super::super::super::root_paths::TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILD;

#[test]
fn runtime_15_typed_error_status_doc_paths_child_inventory_is_child_backed() {
    assert_typed_error_status_doc_paths_child_inventory_parent_mounts_children();
}

pub(super) fn assert_typed_error_status_doc_paths_child_inventory_parent_mounts_children() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILD);

    assert_contains_all(
        "typed-error status-doc paths child-inventory parent mounts focused children",
        &parent,
        &[
            "#[path = \"inventory/guard_children.rs\"]",
            "#[path = \"inventory/paths_children.rs\"]",
            "#[path = \"inventory/source_helper_children.rs\"]",
            "#[path = \"inventory/split_layout.rs\"]",
            "#[path = \"inventory/status_current_children.rs\"]",
            "pub(in super::super) use guard_children::TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN;",
            "pub(in super::super) use paths_children::TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN;",
            "pub(in super::super) use source_helper_children::TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN;",
            "pub(in super::super) use status_current_children::TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN;",
        ],
    );
    super::parent_backflow::assert_typed_error_status_doc_paths_child_inventory_parent_has_no_moved_checks(
        &parent,
    );
    super::child_inventory::assert_typed_error_status_doc_paths_child_inventory_direct_child_inventory(
    );
    super::status_mirrors::assert_typed_error_status_doc_paths_child_inventory_status_is_current();
    super::budgets::assert_typed_error_status_doc_paths_child_inventory_children_line_budgets(
        &parent,
    );
}
