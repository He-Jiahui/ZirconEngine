use super::ExpectedStatusOutputSlice;

#[path = "runtime_09/baseline.rs"]
mod baseline;
#[path = "runtime_09/layout_pipeline.rs"]
mod layout_pipeline;
#[path = "runtime_09/legacy_renames.rs"]
mod legacy_renames;

pub(super) const RUNTIME_09_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    baseline::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_09_LEGACY_RENAME_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = legacy_renames::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_09_LAYOUT_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = layout_pipeline::EXPECTED_STATUS_OUTPUT_SLICES;
