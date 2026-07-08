use super::*;

#[path = "delegation/budgets.rs"]
mod budgets;
#[path = "delegation/child_inventory.rs"]
mod child_inventory;
#[path = "delegation/child_tree.rs"]
mod child_tree;
#[path = "delegation/status_current.rs"]
mod status_current;
#[path = "delegation/status_doc_parent.rs"]
mod status_doc_parent;
#[path = "delegation/typed_error_parent.rs"]
mod typed_error_parent;

pub(super) use child_inventory::*;

#[test]
fn runtime_15_typed_error_status_docs_are_folder_backed() {
    typed_error_parent::assert_typed_error_parent_delegates_status_docs();
    status_doc_parent::assert_typed_error_status_doc_parent_delegates_children();
    child_tree::assert_typed_error_status_doc_children_own_delegated_assertions();
    budgets::assert_typed_error_status_doc_delegation_budgets_are_current();
    assert_typed_error_status_docs_are_synced();
}

#[test]
fn runtime_15_typed_error_status_doc_delegation_is_child_backed() {
    status_current::assert_typed_error_status_doc_delegation_is_child_backed();
}
