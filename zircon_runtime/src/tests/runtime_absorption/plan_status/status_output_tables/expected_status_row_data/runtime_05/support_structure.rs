use super::ExpectedStatusOutputSlice;

#[path = "support_structure/plan_status_modules.rs"]
mod plan_status_modules;
#[path = "support_structure/status_output_splits.rs"]
mod status_output_splits;

pub(super) const RUNTIME_05_PLAN_STATUS_MODULE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = plan_status_modules::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_STATUS_OUTPUT_SPLIT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = status_output_splits::EXPECTED_STATUS_OUTPUT_SLICES;
