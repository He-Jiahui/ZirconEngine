pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "ui_tests_first/accessibility_surface_rows.rs"]
mod accessibility_surface_rows;
#[path = "ui_tests_first/architecture_shared_rows.rs"]
mod architecture_shared_rows;
#[path = "ui_tests_first/row_data_owner.rs"]
mod row_data_owner;
#[path = "ui_tests_first/template_input_rows.rs"]
mod template_input_rows;

#[path = "ui_tests_first/route_mirrors.rs"]
mod route_mirrors;


pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    architecture_shared_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ACCESSIBILITY_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    accessibility_surface_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TEMPLATE_INPUT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    template_input_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
