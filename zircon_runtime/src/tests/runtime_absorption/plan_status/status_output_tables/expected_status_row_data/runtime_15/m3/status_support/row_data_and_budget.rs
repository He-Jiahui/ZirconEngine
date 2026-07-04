use super::ExpectedStatusOutputSlice;

#[path = "row_data_and_budget/hub_editor_support.rs"]
mod hub_editor_support;
#[path = "row_data_and_budget/m3_m4_row_data.rs"]
mod m3_m4_row_data;
#[path = "row_data_and_budget/render_shader_support.rs"]
mod render_shader_support;
#[path = "row_data_and_budget/runtime_row_data.rs"]
mod runtime_row_data;
#[path = "row_data_and_budget/test_file_budget.rs"]
mod test_file_budget;

pub(super) const TEST_FILE_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    test_file_budget::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_row_data::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const HUB_EDITOR_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    hub_editor_support::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RENDER_SHADER_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = render_shader_support::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const M3_M4_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    m3_m4_row_data::EXPECTED_STATUS_OUTPUT_SLICES;
