use super::*;

#[path = "runtime_15_asset_budget_row_data/budgets.rs"]
mod budgets;
#[path = "runtime_15_asset_budget_row_data/delegation.rs"]
mod delegation;
#[path = "runtime_15_asset_budget_row_data/export_chain.rs"]
mod export_chain;
#[path = "runtime_15_asset_budget_row_data/root_child_rows.rs"]
mod root_child_rows;
#[path = "runtime_15_asset_budget_row_data/root_inventory.rs"]
mod root_inventory;
#[path = "runtime_15_asset_budget_row_data/root_owner_paths.rs"]
mod root_owner_paths;
#[path = "runtime_15_asset_budget_row_data/root_paths.rs"]
mod root_paths;
#[path = "runtime_15_asset_budget_row_data/root_statuses.rs"]
mod root_statuses;
#[path = "runtime_15_asset_budget_row_data/row_ownership.rs"]
mod row_ownership;
#[path = "runtime_15_asset_budget_row_data/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_owner_paths::*;
pub(super) use root_paths::*;
pub(super) use root_statuses::*;

pub(super) fn asset_budget_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in ASSET_BUDGET_ROW_DATA_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
