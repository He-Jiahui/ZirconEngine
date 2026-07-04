use super::*;

#[path = "child_ownership/budgets.rs"]
mod budgets;
#[path = "child_ownership/delegation.rs"]
mod delegation;
#[path = "child_ownership/review_guards.rs"]
mod review_guards;
#[path = "child_ownership/root_child_rows.rs"]
mod root_child_rows;
#[path = "child_ownership/root_inventory.rs"]
mod root_inventory;
#[path = "child_ownership/root_paths.rs"]
mod root_paths;
#[path = "child_ownership/root_sources.rs"]
mod root_sources;
#[path = "child_ownership/root_statuses.rs"]
mod root_statuses;
#[path = "child_ownership/status_mirrors.rs"]
mod status_mirrors;
#[path = "child_ownership/structure_subtree.rs"]
mod structure_subtree;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;
pub(super) use root_statuses::*;

#[test]
fn runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner() {
    let sources = typed_error_child_ownership_sources();

    delegation::assert_typed_error_child_ownership_is_folder_backed(&sources);
    structure_subtree::assert_typed_error_structure_subtree_is_child_owned(&sources);
    review_guards::assert_typed_error_review_guards_are_preserved(&sources);
    budgets::assert_typed_error_child_ownership_budgets_are_focused(&sources);
    assert_typed_error_status_docs_are_synced();
}
