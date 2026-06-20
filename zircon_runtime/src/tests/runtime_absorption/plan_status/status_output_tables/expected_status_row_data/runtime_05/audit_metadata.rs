use super::ExpectedStatusOutputSlice;

#[path = "audit_metadata/plan_coverage_rows.rs"]
mod plan_coverage_rows;
#[path = "audit_metadata/runtime_02_03_rows.rs"]
mod runtime_02_03_rows;
#[path = "audit_metadata/runtime_07_rows.rs"]
mod runtime_07_rows;

pub(super) const RUNTIME_05_AUDIT_PLAN_COVERAGE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = plan_coverage_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_AUDIT_RUNTIME_02_03_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_02_03_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_AUDIT_RUNTIME_07_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_07_rows::EXPECTED_STATUS_OUTPUT_SLICES;
