use super::ExpectedStatusOutputSlice;

#[path = "runtime_10_13/runtime_10.rs"]
mod runtime_10;
#[path = "runtime_10_13/runtime_11.rs"]
mod runtime_11;
#[path = "runtime_10_13/runtime_12.rs"]
mod runtime_12;
#[path = "runtime_10_13/runtime_13.rs"]
mod runtime_13;

pub(super) const RUNTIME_10_DYNAMIC_API_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_10::RUNTIME_10_DYNAMIC_API_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_10_SESSION_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_10::RUNTIME_10_SESSION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_10_UI_CONTRACT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_10::RUNTIME_10_UI_CONTRACT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_11_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_11::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_12_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_12::RUNTIME_12_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_12_ACTION_MAPPING_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_12::RUNTIME_12_ACTION_MAPPING_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_12_GAMEPAD_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_12::RUNTIME_12_GAMEPAD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_12_HOST_RECORDING_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_12::RUNTIME_12_HOST_RECORDING_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_13_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_13::EXPECTED_STATUS_OUTPUT_SLICES;
