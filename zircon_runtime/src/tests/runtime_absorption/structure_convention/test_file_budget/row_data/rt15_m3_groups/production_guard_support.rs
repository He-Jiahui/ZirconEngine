use super::*;

#[path = "production_guard_support/inventory_row_data.rs"]
mod inventory_row_data;
#[path = "production_guard_support/row_data_children.rs"]
mod row_data_children;

#[test]
fn runtime_15_production_guard_support_row_data_children_are_child_owned() {
    row_data_children::assert_runtime_15_production_guard_support_row_data_children_are_child_owned(
    );
}

#[test]
fn runtime_15_m3_child_groups_inventory_row_data_is_child_owned() {
    inventory_row_data::assert_runtime_15_m3_child_groups_inventory_row_data_is_child_owned();
}
