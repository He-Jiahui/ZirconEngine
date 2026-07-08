pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "runtime_structure_tests/core_runtime_rows.rs"]
mod core_runtime_rows;
#[path = "runtime_structure_tests/root_route_rows.rs"]
mod root_route_rows;
#[path = "runtime_structure_tests/row_data_owner.rs"]
mod row_data_owner;
#[path = "runtime_structure_tests/runtime_absorption_core_rows.rs"]
mod runtime_absorption_core_rows;
#[path = "runtime_structure_tests/runtime_absorption_platform_rows.rs"]
mod runtime_absorption_platform_rows;
#[path = "runtime_structure_tests/test_guard_rows.rs"]
mod test_guard_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    core_runtime_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROOT_ROUTE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ABSORPTION_CORE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    runtime_absorption_core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ABSORPTION_PLATFORM_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    runtime_absorption_platform_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TEST_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    test_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
