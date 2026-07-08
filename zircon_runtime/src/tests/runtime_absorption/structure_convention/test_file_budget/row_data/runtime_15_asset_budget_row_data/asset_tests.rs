use super::*;

#[path = "asset_tests/child_rows.rs"]
mod child_rows;
#[path = "asset_tests/export_chain.rs"]
mod export_chain;
#[path = "asset_tests/folder_backed.rs"]
mod folder_backed;
#[path = "asset_tests/status_mirrors.rs"]
mod status_mirrors;

#[test]
fn runtime_15_asset_budget_asset_tests_row_data_is_child_backed() {
    child_rows::assert_asset_tests_child_rows_are_route_owned();
    export_chain::assert_asset_tests_exports_are_current();
    status_mirrors::assert_asset_tests_row_data_status_is_current();
}

#[test]
fn runtime_15_asset_budget_asset_tests_guard_is_folder_backed() {
    folder_backed::assert_asset_tests_guard_is_folder_backed();
    status_mirrors::assert_asset_tests_guard_status_is_current();
}
