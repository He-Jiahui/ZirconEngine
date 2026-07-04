use super::*;

#[path = "runtime_15_m2_row_data_children/budgets.rs"]
mod budgets;
#[path = "runtime_15_m2_row_data_children/delegation.rs"]
mod delegation;
#[path = "runtime_15_m2_row_data_children/root_child_rows.rs"]
mod root_child_rows;
#[path = "runtime_15_m2_row_data_children/root_inventory.rs"]
mod root_inventory;
#[path = "runtime_15_m2_row_data_children/root_owner_paths.rs"]
mod root_owner_paths;
#[path = "runtime_15_m2_row_data_children/root_paths.rs"]
mod root_paths;
#[path = "runtime_15_m2_row_data_children/root_statuses.rs"]
mod root_statuses;
#[path = "runtime_15_m2_row_data_children/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_m2_row_data_children/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_owner_paths::*;
pub(super) use root_paths::*;
pub(super) use root_statuses::*;

pub(super) fn m2_row_data_children_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in M2_ROW_DATA_CHILDREN_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
