use super::*;

#[path = "rt15_m3_groups/budgets.rs"]
mod budgets;
#[path = "rt15_m3_groups/delegation.rs"]
mod delegation;
#[path = "rt15_m3_groups/exports.rs"]
mod exports;
#[path = "rt15_m3_groups/module_convention_status.rs"]
mod module_convention_status;
#[path = "rt15_m3_groups/production_guard_runtime_row_data.rs"]
mod production_guard_runtime_row_data;
#[path = "rt15_m3_groups/production_guard_support.rs"]
mod production_guard_support;
#[path = "rt15_m3_groups/review_status_sync.rs"]
mod review_status_sync;
#[path = "rt15_m3_groups/root_child_rows.rs"]
mod root_child_rows;
#[path = "rt15_m3_groups/root_inventory.rs"]
mod root_inventory;
#[path = "rt15_m3_groups/root_owner_paths.rs"]
mod root_owner_paths;
#[path = "rt15_m3_groups/root_paths.rs"]
mod root_paths;
#[path = "rt15_m3_groups/root_statuses.rs"]
mod root_statuses;
#[path = "rt15_m3_groups/row_ownership.rs"]
mod row_ownership;
#[path = "rt15_m3_groups/status_mirrors.rs"]
mod status_mirrors;
#[path = "rt15_m3_groups/ui_tests_first.rs"]
mod ui_tests_first;
#[path = "rt15_m3_groups/ui_tests_second.rs"]
mod ui_tests_second;

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

pub(super) fn m3_child_group_core_status_source_blob() -> String {
    [
        read_runtime_src(ROOT_STATUSES_CORE_PATH),
        read_runtime_src(ROOT_STATUSES_CORE_BASE_PATH),
        read_runtime_src(ROOT_STATUSES_CORE_INVENTORY_PATH),
        read_runtime_src(ROOT_STATUSES_CORE_PRODUCTION_PATH),
    ]
    .join("\n")
}
