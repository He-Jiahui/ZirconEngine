use super::*;

#[path = "module_convention_status/export_chain.rs"]
mod export_chain;
#[path = "module_convention_status/row_groups.rs"]
mod row_groups;
#[path = "module_convention_status/status_current.rs"]
mod status_current;

#[test]
fn runtime_15_module_convention_status_row_data_owner_is_child_backed() {
    row_groups::assert_module_convention_status_parent_delegates_to_children();
    export_chain::assert_module_convention_status_export_chain_is_current();
    status_current::assert_module_convention_status_status_mirrors_are_current();
}
