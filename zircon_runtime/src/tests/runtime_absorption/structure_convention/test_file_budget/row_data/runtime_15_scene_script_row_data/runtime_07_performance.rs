use super::*;

#[path = "runtime_07_performance/child_rows.rs"]
mod child_rows;
#[path = "runtime_07_performance/export_chain.rs"]
mod export_chain;
#[path = "runtime_07_performance/folder_backed.rs"]
mod folder_backed;
#[path = "runtime_07_performance/status_mirrors.rs"]
mod status_mirrors;

#[test]
fn runtime_15_scene_script_runtime_07_performance_row_data_is_child_backed() {
    child_rows::assert_runtime_07_performance_child_rows_are_route_owned();
    export_chain::assert_runtime_07_performance_exports_are_current();
    status_mirrors::assert_runtime_07_performance_row_data_status_is_current();
}

#[test]
fn runtime_15_scene_script_runtime_07_performance_guard_is_folder_backed() {
    folder_backed::assert_runtime_07_performance_guard_is_folder_backed();
    status_mirrors::assert_runtime_07_performance_guard_status_is_current();
}
