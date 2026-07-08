use super::*;

#[path = "typed_error_structure/budgets.rs"]
mod budgets;
#[path = "typed_error_structure/delegation.rs"]
mod delegation;
#[path = "typed_error_structure/folder_backed.rs"]
mod folder_backed;
#[path = "typed_error_structure/paths.rs"]
mod paths;
#[path = "typed_error_structure/row_routes.rs"]
mod row_routes;
#[path = "typed_error_structure/status_mirrors.rs"]
mod status_mirrors;

pub(super) use paths::*;

#[test]
fn runtime_15_review_guard_typed_error_structure_assertions_guard_is_folder_backed() {
    folder_backed::assert_typed_error_structure_assertions_guard_is_folder_backed();
    status_mirrors::assert_typed_error_structure_assertions_guard_status_is_current();
}
