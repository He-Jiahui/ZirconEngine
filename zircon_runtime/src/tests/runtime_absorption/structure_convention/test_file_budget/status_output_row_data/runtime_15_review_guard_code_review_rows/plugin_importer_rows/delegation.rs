use super::*;

#[test]
fn runtime_15_plugin_importer_rows_row_data_owner_is_child_backed() {
    row_children::assert_plugin_importer_row_data_children_are_current();
    row_data_status::assert_plugin_importer_row_data_owner_status_row_is_current();
    status_mirrors::assert_plugin_importer_row_data_status_mirrors_are_current();
}
