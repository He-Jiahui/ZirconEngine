use super::ExpectedStatusOutputSlice;

#[path = "runtime_06_09/runtime_06.rs"]
mod runtime_06;
#[path = "runtime_06_09/runtime_07.rs"]
mod runtime_07;
#[path = "runtime_06_09/runtime_08.rs"]
mod runtime_08;
#[path = "runtime_06_09/runtime_09.rs"]
mod runtime_09;

pub(super) const RUNTIME_06_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_06::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_07_PERFORMANCE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_07::RUNTIME_07_PERFORMANCE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_07_ASSET_RENDER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_07::RUNTIME_07_ASSET_RENDER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_07_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_07::RUNTIME_07_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_07_OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_07::RUNTIME_07_OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_08_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_08::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_09_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    runtime_09::RUNTIME_09_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_09_LEGACY_RENAME_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_09::RUNTIME_09_LEGACY_RENAME_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_09_LAYOUT_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_09::RUNTIME_09_LAYOUT_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES;
