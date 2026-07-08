use super::*;

#[path = "module_layout/budgets.rs"]
mod budgets;
#[path = "module_layout/child_summaries.rs"]
mod child_summaries;
#[path = "module_layout/delegation.rs"]
mod delegation;
#[path = "module_layout/root_child_rows.rs"]
mod root_child_rows;
#[path = "module_layout/root_inventory.rs"]
mod root_inventory;
#[path = "module_layout/root_owner_paths.rs"]
mod root_owner_paths;
#[path = "module_layout/root_paths.rs"]
mod root_paths;
#[path = "module_layout/root_statuses.rs"]
mod root_statuses;
#[path = "module_layout/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_owner_paths::*;
pub(super) use root_paths::*;
pub(super) use root_statuses::*;

pub(super) fn module_layout_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in MODULE_LAYOUT_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
