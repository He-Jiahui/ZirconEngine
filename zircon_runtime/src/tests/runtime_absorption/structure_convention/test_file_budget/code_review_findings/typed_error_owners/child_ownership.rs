use super::*;

#[path = "ownership/budgets.rs"]
mod budgets;
#[path = "ownership/delegation.rs"]
mod delegation;
#[path = "ownership/review_guards.rs"]
mod review_guards;
#[path = "ownership/root_child_rows.rs"]
mod root_child_rows;
#[path = "ownership/root_inventory.rs"]
mod root_inventory;
#[path = "ownership/root_paths.rs"]
mod root_paths;
#[path = "ownership/root_sources.rs"]
mod root_sources;
#[path = "ownership/structure_subtree.rs"]
mod structure_subtree;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;

#[test]
fn runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner() {
    let sources = typed_error_child_ownership_sources();

    delegation::assert_typed_error_child_ownership_is_folder_backed(&sources);
    structure_subtree::assert_typed_error_structure_subtree_is_child_owned(&sources);
    review_guards::assert_typed_error_review_guards_are_preserved(&sources);
    budgets::assert_typed_error_child_ownership_budgets_are_focused(&sources);
}
