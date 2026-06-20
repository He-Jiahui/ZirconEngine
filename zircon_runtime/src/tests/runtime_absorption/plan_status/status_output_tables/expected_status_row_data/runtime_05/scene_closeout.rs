use super::ExpectedStatusOutputSlice;

#[path = "scene_closeout/dynamic_scene_rows.rs"]
mod dynamic_scene_rows;
#[path = "scene_closeout/full_scene_gate_rows.rs"]
mod full_scene_gate_rows;
#[path = "scene_closeout/source_guard_rows.rs"]
mod source_guard_rows;

pub(super) const RUNTIME_05_SCENE_CLOSEOUT_DYNAMIC_SCENE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = dynamic_scene_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_SCENE_CLOSEOUT_FULL_SCENE_GATE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = full_scene_gate_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_05_SCENE_CLOSEOUT_SOURCE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = source_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES;
