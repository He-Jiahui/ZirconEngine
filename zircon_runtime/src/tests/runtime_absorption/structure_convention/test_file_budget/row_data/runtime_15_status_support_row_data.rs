use super::*;

#[path = "rt15_status_support/anchor_mirror.rs"]
mod anchor_mirror;
#[path = "rt15_status_support/budgets.rs"]
mod budgets;
#[path = "rt15_status_support/delegation.rs"]
mod delegation;
#[path = "rt15_status_support/expected_slice_maps.rs"]
mod expected_slice_maps;
#[path = "rt15_status_support/expected_slice_review_guard_structure.rs"]
mod expected_slice_review_guard_structure;
#[path = "rt15_status_support/expected_slice_route_metadata.rs"]
mod expected_slice_route_metadata;
#[path = "rt15_status_support/expected_slice_status_support_maps.rs"]
mod expected_slice_status_support_maps;
#[path = "rt15_status_support/expected_slice_status_support_route_guard_rows.rs"]
mod expected_slice_status_support_route_guard_rows;
#[path = "rt15_status_support/expected_slice_structure_support.rs"]
mod expected_slice_structure_support;
#[path = "rt15_status_support/expected_slice_top_level_support.rs"]
mod expected_slice_top_level_support;
#[path = "rt15_status_support/export_chain.rs"]
mod export_chain;
#[path = "rt15_status_support/root_child_rows.rs"]
mod root_child_rows;
#[path = "rt15_status_support/root_inventory.rs"]
mod root_inventory;
#[path = "rt15_status_support/root_owner_paths.rs"]
mod root_owner_paths;
#[path = "rt15_status_support/root_paths.rs"]
mod root_paths;
#[path = "rt15_status_support/root_statuses.rs"]
mod root_statuses;
#[path = "rt15_status_support/row_data_and_budget.rs"]
mod row_data_and_budget;
#[path = "rt15_status_support/row_ownership.rs"]
mod row_ownership;
#[path = "rt15_status_support/runtime_index_anchors.rs"]
mod runtime_index_anchors;
#[path = "rt15_status_support/status_mirrors.rs"]
mod status_mirrors;

pub(super) use root_child_rows::*;
pub(super) use root_owner_paths::*;
pub(super) use root_paths::*;
pub(super) use root_statuses::*;

pub(super) fn status_support_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_SUPPORT_ROW_DATA_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
