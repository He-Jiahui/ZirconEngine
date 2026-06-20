use super::ExpectedStatusOutputSlice;

#[path = "cargo_gates/early_rows.rs"]
mod early_rows;
#[path = "cargo_gates/late_rows.rs"]
mod late_rows;

pub(super) const RUNTIME_05_CARGO_EARLY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = early_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_CARGO_LATE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = late_rows::EXPECTED_STATUS_OUTPUT_SLICES;
