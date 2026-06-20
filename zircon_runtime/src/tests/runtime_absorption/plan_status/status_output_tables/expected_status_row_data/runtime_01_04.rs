use super::ExpectedStatusOutputSlice;

#[path = "runtime_01_04/runtime_01.rs"]
mod runtime_01;
#[path = "runtime_01_04/runtime_02.rs"]
mod runtime_02;
#[path = "runtime_01_04/runtime_03.rs"]
mod runtime_03;
#[path = "runtime_01_04/runtime_04.rs"]
mod runtime_04;

pub(super) const RUNTIME_01_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_01::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_02_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_02::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_03_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_03::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_04_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_04::EXPECTED_STATUS_OUTPUT_SLICES;
