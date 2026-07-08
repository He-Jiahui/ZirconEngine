use super::*;

#[path = "typed_error_status/budgets.rs"]
mod budgets;
#[path = "typed_error_status/delegation.rs"]
mod delegation;
#[path = "typed_error_status/folder_backed.rs"]
mod folder_backed;
#[path = "typed_error_status/paths.rs"]
mod paths;
#[path = "typed_error_status/row_routes.rs"]
mod row_routes;
#[path = "typed_error_status/status_mirrors.rs"]
mod status_mirrors;

pub(super) use paths::*;

#[test]
fn runtime_15_review_guard_typed_error_status_docs_guard_is_folder_backed() {
    folder_backed::assert_typed_error_status_docs_guard_is_folder_backed();
    status_mirrors::assert_typed_error_status_doc_guard_status_is_current();
}
