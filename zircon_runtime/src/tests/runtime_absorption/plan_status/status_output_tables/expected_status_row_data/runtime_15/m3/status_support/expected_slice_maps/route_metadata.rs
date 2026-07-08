type Slice = super::Slice;

#[path = "route_metadata/child_owner_budget_rows.rs"]
mod child_owner_budget_rows;
#[path = "route_metadata/child_owner_guard_rows.rs"]
mod child_owner_guard_rows;
#[path = "route_metadata/naming_boundary_rows.rs"]
mod naming_boundary_rows;
#[path = "route_metadata/row_data_owner_rows.rs"]
mod row_data_owner_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    naming_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    naming_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    naming_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    naming_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
    child_owner_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    child_owner_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    child_owner_budget_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    child_owner_budget_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    child_owner_budget_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
];
