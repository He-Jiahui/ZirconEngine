use super::*;

#[test]
fn runtime_15_typed_error_structure_rows_row_data_owner_is_child_backed() {
    row_groups::assert_typed_error_structure_row_groups_are_child_backed();
    status_doc_paths::assert_status_doc_paths_rows_are_child_backed();
    status_mirrors::assert_typed_error_structure_row_data_status_is_current();
}
