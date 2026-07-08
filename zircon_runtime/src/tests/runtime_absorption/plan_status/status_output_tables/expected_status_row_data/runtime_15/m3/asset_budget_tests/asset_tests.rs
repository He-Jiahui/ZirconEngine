pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "asset_tests/asset_resource_rows.rs"]
mod asset_resource_rows;
#[path = "asset_tests/project_rows.rs"]
mod project_rows;
#[path = "asset_tests/row_data_owner.rs"]
mod row_data_owner;
#[path = "asset_tests/ui_pipeline_rows.rs"]
mod ui_pipeline_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    project_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ASSET_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    asset_resource_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const UI_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    ui_pipeline_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
