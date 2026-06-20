use super::ExpectedStatusOutputSlice;

#[path = "runtime_12/action_mapping.rs"]
mod action_mapping;
#[path = "runtime_12/baseline.rs"]
mod baseline;
#[path = "runtime_12/gamepad.rs"]
mod gamepad;
#[path = "runtime_12/host_recording.rs"]
mod host_recording;

pub(super) const RUNTIME_12_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    baseline::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_12_ACTION_MAPPING_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = action_mapping::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_12_GAMEPAD_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    gamepad::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_12_HOST_RECORDING_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = host_recording::EXPECTED_STATUS_OUTPUT_SLICES;
