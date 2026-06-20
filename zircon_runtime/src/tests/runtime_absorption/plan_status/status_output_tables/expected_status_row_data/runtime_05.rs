use super::ExpectedStatusOutputSlice;

#[path = "runtime_05/audit_metadata.rs"]
mod audit_metadata;
#[path = "runtime_05/baseline.rs"]
mod baseline;
#[path = "runtime_05/cargo_gates.rs"]
mod cargo_gates;
#[path = "runtime_05/cross_runtime_rows.rs"]
mod cross_runtime_rows;
#[path = "runtime_05/scene_closeout.rs"]
mod scene_closeout;
#[path = "runtime_05/support_structure.rs"]
mod support_structure;

pub(super) const RUNTIME_05_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    baseline::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_CROSS_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = cross_runtime_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_SUPPORT_STRUCTURE_PLAN_STATUS_MODULE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    support_structure::RUNTIME_05_PLAN_STATUS_MODULE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_SUPPORT_STRUCTURE_STATUS_OUTPUT_SPLIT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    support_structure::RUNTIME_05_STATUS_OUTPUT_SPLIT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_SCENE_CLOSEOUT_DYNAMIC_SCENE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    scene_closeout::RUNTIME_05_SCENE_CLOSEOUT_DYNAMIC_SCENE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_SCENE_CLOSEOUT_FULL_SCENE_GATE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    scene_closeout::RUNTIME_05_SCENE_CLOSEOUT_FULL_SCENE_GATE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_SCENE_CLOSEOUT_SOURCE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    scene_closeout::RUNTIME_05_SCENE_CLOSEOUT_SOURCE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_CARGO_EARLY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    cargo_gates::RUNTIME_05_CARGO_EARLY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_CARGO_LATE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = cargo_gates::RUNTIME_05_CARGO_LATE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_AUDIT_PLAN_COVERAGE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    audit_metadata::RUNTIME_05_AUDIT_PLAN_COVERAGE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_AUDIT_RUNTIME_02_03_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    audit_metadata::RUNTIME_05_AUDIT_RUNTIME_02_03_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_AUDIT_RUNTIME_07_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    audit_metadata::RUNTIME_05_AUDIT_RUNTIME_07_EXPECTED_STATUS_OUTPUT_SLICES;
