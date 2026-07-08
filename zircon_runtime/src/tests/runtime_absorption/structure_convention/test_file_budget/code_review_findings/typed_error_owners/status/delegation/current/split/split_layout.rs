use super::super::super::super::super::super::super::*;
use super::super::super::super::TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD;

#[test]
fn runtime_15_typed_error_status_doc_delegation_status_current_split_layout_guard_is_folder_backed()
{
    assert_typed_error_status_doc_delegation_status_current_split_layout_is_folder_backed();
    super::status_mirrors::assert_typed_error_status_doc_delegation_status_current_split_layout_guard_status_is_current();
}

fn assert_typed_error_status_doc_delegation_status_current_split_layout_is_folder_backed() {
    let parent =
        read_runtime_src(TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD);

    assert_contains_all(
        "typed-error status-doc delegation status-current split-layout mounts focused children",
        &parent,
        &[
            "#[path = \"split/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"split/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"split/parent_backflow.rs\"]",
            "mod parent_backflow;",
            "#[path = \"split/parent_mounts.rs\"]",
            "mod parent_mounts;",
            "#[path = \"split/sources.rs\"]",
            "mod sources;",
            "#[path = \"split/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"split/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    super::parent_backflow::assert_typed_error_status_doc_delegation_status_current_split_layout_parent_has_no_moved_checks(
        &parent,
    );
    super::child_inventory::assert_typed_error_status_doc_delegation_status_current_split_layout_child_inventory();
    super::budgets::assert_typed_error_status_doc_delegation_status_current_split_layout_children_line_budgets(
        &parent,
    );
}
