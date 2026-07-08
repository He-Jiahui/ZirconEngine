use super::*;

#[test]
fn runtime_15_review_guard_typed_error_structure_assertions_row_data_is_folder_backed() {
    row_routes::assert_typed_error_structure_assertion_row_routes_are_child_backed();
    status_mirrors::assert_typed_error_structure_assertions_row_data_status_is_current();
}
