pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "ui_tests_second/asset_style_rows.rs"]
mod asset_style_rows;
#[path = "ui_tests_second/component_boundary_rows.rs"]
mod component_boundary_rows;
#[path = "ui_tests_second/row_data_owner.rs"]
mod row_data_owner;
#[path = "ui_tests_second/runtime_input_rows.rs"]
mod runtime_input_rows;

#[path = "ui_tests_second/route_mirrors.rs"]
mod route_mirrors;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    component_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ASSET_STYLE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    asset_style_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_INPUT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    runtime_input_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
