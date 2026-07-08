use super::ExpectedStatusOutputSlice;

#[path = "core_and_evidence/child_group_inventory_rows.rs"]
mod child_group_inventory_rows;
#[path = "core_and_evidence/child_group_row_data_rows.rs"]
mod child_group_row_data_rows;
#[path = "core_and_evidence/evidence_anchor_rows.rs"]
mod evidence_anchor_rows;
#[path = "core_and_evidence/production_file_budget_rows.rs"]
mod production_file_budget_rows;

pub(super) const PRODUCTION_FILE_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = production_file_budget_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EVIDENCE_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    evidence_anchor_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = child_group_row_data_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_INVENTORY_ROOT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    child_group_inventory_rows::ROOT_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_INVENTORY_OWNER_PATH_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    child_group_inventory_rows::OWNER_PATH_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_INVENTORY_ROOT_PATH_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    child_group_inventory_rows::ROOT_PATH_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CHILD_GROUP_INVENTORY_GUARD_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    child_group_inventory_rows::GUARD_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES;
