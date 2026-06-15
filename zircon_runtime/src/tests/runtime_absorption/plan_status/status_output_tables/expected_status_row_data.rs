use super::ExpectedStatusOutputSlice;

#[path = "expected_status_row_data/runtime_01_04.rs"]
mod runtime_01_04;
#[path = "expected_status_row_data/runtime_05.rs"]
mod runtime_05;
#[path = "expected_status_row_data/runtime_06_09.rs"]
mod runtime_06_09;
#[path = "expected_status_row_data/runtime_10_13.rs"]
mod runtime_10_13;
#[path = "expected_status_row_data/runtime_14.rs"]
mod runtime_14;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICE_GROUPS: &[&[ExpectedStatusOutputSlice]] = &[
    runtime_01_04::EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_14::EXPECTED_STATUS_OUTPUT_SLICES,
];
