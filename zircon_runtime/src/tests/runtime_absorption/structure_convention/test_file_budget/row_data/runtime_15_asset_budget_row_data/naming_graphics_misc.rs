use super::*;

#[path = "naming_graphics_misc/child_rows.rs"]
mod child_rows;
#[path = "naming_graphics_misc/export_chain.rs"]
mod export_chain;
#[path = "naming_graphics_misc/folder_backed.rs"]
mod folder_backed;
#[path = "naming_graphics_misc/status_mirrors.rs"]
mod status_mirrors;

#[test]
fn runtime_15_asset_budget_naming_graphics_misc_row_data_is_child_backed() {
    child_rows::assert_naming_graphics_misc_child_rows_are_route_owned();
    export_chain::assert_naming_graphics_misc_exports_are_current();
    status_mirrors::assert_naming_graphics_misc_row_data_status_is_current();
}

#[test]
fn runtime_15_asset_budget_naming_graphics_misc_guard_is_folder_backed() {
    folder_backed::assert_naming_graphics_misc_guard_is_folder_backed();
    status_mirrors::assert_naming_graphics_misc_guard_status_is_current();
}
