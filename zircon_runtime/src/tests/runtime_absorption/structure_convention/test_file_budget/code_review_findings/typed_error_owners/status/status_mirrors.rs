#[path = "mirrors/budgets.rs"]
mod budgets;
#[path = "mirrors/child_inventory.rs"]
mod child_inventory;
#[path = "mirrors/folder_backed_status.rs"]
mod folder_backed_status;
#[path = "mirrors/status_current.rs"]
mod status_current;

pub(super) use child_inventory::*;

#[test]
fn runtime_15_typed_error_status_docs_folder_backed_status_is_current() {
    folder_backed_status::assert_typed_error_status_docs_folder_backed_status_is_current();
    budgets::assert_typed_error_status_mirror_child_budgets_are_current();
}

#[test]
fn runtime_15_typed_error_status_doc_status_mirrors_are_child_backed() {
    status_current::assert_typed_error_status_mirrors_are_child_backed();
}
