use super::ExpectedStatusOutputSlice;

#[path = "module_convention_status/audit_rows.rs"]
mod audit_rows;
#[path = "module_convention_status/frontmatter_and_gate_rows.rs"]
mod frontmatter_and_gate_rows;
#[path = "module_convention_status/row_data_owner.rs"]
mod row_data_owner;
#[path = "module_convention_status/status_rows.rs"]
mod status_rows;
#[path = "module_convention_status/structure_guard_rows.rs"]
mod structure_guard_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    status_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FRONTMATTER_AND_GATE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = frontmatter_and_gate_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    structure_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const AUDIT_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    audit_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
