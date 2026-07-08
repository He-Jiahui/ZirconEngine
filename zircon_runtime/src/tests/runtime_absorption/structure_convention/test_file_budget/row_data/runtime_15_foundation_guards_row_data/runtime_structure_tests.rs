use super::*;

#[path = "runtime_structure_tests/child_rows.rs"]
mod child_rows;
#[path = "runtime_structure_tests/export_chain.rs"]
mod export_chain;
#[path = "runtime_structure_tests/folder_backed.rs"]
mod folder_backed;
#[path = "runtime_structure_tests/status_mirrors.rs"]
mod status_mirrors;

#[test]
fn runtime_15_foundation_guards_runtime_structure_row_data_is_child_backed() {
    child_rows::assert_runtime_structure_child_rows_are_route_owned();
    export_chain::assert_runtime_structure_exports_are_current();
    status_mirrors::assert_runtime_structure_row_data_status_is_current();
}

#[test]
fn runtime_15_foundation_guards_runtime_structure_guard_is_folder_backed() {
    folder_backed::assert_runtime_structure_guard_is_folder_backed();
    status_mirrors::assert_runtime_structure_guard_status_is_current();
}
