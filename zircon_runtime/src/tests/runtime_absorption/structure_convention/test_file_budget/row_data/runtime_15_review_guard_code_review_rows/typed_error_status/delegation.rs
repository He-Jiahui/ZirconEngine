use super::*;

#[test]
fn runtime_15_review_guard_typed_error_status_docs_row_data_is_folder_backed() {
    row_routes::assert_typed_error_status_doc_row_routes_are_child_backed();
    status_mirrors::assert_typed_error_status_doc_row_data_status_is_current();
}
