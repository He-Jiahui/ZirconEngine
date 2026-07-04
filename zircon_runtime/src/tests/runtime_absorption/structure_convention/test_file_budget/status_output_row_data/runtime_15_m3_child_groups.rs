use super::*;

#[path = "runtime_15_m3_child_groups/budgets.rs"]
mod budgets;
#[path = "runtime_15_m3_child_groups/delegation.rs"]
mod delegation;
#[path = "runtime_15_m3_child_groups/exports.rs"]
mod exports;
#[path = "runtime_15_m3_child_groups/production_guard_runtime_row_data.rs"]
mod production_guard_runtime_row_data;
#[path = "runtime_15_m3_child_groups/production_guard_support.rs"]
mod production_guard_support;
#[path = "runtime_15_m3_child_groups/root_child_rows.rs"]
mod root_child_rows;
#[path = "runtime_15_m3_child_groups/root_inventory.rs"]
mod root_inventory;
#[path = "runtime_15_m3_child_groups/root_owner_paths.rs"]
mod root_owner_paths;
#[path = "runtime_15_m3_child_groups/root_paths.rs"]
mod root_paths;
#[path = "runtime_15_m3_child_groups/root_statuses.rs"]
mod root_statuses;
#[path = "runtime_15_m3_child_groups/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_m3_child_groups/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_owner_paths::*;
pub(super) use root_paths::*;
pub(super) use root_statuses::*;

pub(super) fn m3_child_group_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in M3_CHILD_GROUP_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
