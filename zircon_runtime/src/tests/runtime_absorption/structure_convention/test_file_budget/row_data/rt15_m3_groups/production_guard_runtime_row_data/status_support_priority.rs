use super::super::*;

#[path = "status_support_priority/child_rows.rs"]
mod child_rows;
#[path = "status_support_priority/export_chain.rs"]
mod export_chain;
#[path = "status_support_priority/folder_backed.rs"]
mod folder_backed;
#[path = "status_support_priority/status_mirrors.rs"]
mod status_mirrors;

pub(super) fn assert_status_support_priority_rows_are_child_backed() {
    child_rows::assert_status_support_priority_child_rows_are_route_owned();
    export_chain::assert_status_support_priority_exports_are_current();
    status_mirrors::assert_status_support_priority_row_data_status_is_current();
}

pub(super) fn assert_status_support_priority_guard_is_folder_backed() {
    folder_backed::assert_status_support_priority_guard_is_folder_backed();
    status_mirrors::assert_status_support_priority_guard_status_is_current();
}
