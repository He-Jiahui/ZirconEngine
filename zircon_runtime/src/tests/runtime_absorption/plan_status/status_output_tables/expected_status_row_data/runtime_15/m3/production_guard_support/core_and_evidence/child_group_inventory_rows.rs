use super::ExpectedStatusOutputSlice;

#[path = "child_group_inventory_rows/guard_inventory_rows.rs"]
mod guard_inventory_rows;
#[path = "child_group_inventory_rows/owner_path_rows.rs"]
mod owner_path_rows;
#[path = "child_group_inventory_rows/root_inventory_rows.rs"]
mod root_inventory_rows;
#[path = "child_group_inventory_rows/root_path_rows.rs"]
mod root_path_rows;

pub(super) const ROOT_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    root_inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const OWNER_PATH_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    owner_path_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROOT_PATH_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    root_path_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const GUARD_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    guard_inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES;
