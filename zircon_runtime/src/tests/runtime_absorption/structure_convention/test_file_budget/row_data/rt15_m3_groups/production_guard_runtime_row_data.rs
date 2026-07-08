use super::*;

#[path = "production_guard_runtime_row_data/child_rows.rs"]
mod child_rows;
#[path = "production_guard_runtime_row_data/export_chain.rs"]
mod export_chain;
#[path = "production_guard_runtime_row_data/status_mirrors.rs"]
mod status_mirrors;
#[path = "production_guard_runtime_row_data/status_support_priority.rs"]
mod status_support_priority;

#[test]
fn runtime_15_production_guard_runtime_row_data_children_are_child_owned() {
    child_rows::assert_runtime_row_data_parent_delegates_to_children();
    status_support_priority::assert_status_support_priority_rows_are_child_backed();
    export_chain::assert_runtime_row_data_export_chain_is_current();
    status_mirrors::assert_runtime_row_data_status_mirrors_are_current();
}

#[test]
fn runtime_15_production_guard_status_support_priority_guard_is_folder_backed() {
    status_support_priority::assert_status_support_priority_guard_is_folder_backed();
}
