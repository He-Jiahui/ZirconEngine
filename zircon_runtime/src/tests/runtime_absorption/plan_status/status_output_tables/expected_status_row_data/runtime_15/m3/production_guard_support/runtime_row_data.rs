use super::ExpectedStatusOutputSlice;

#[path = "runtime_row_data/asset_budget_rows.rs"]
mod asset_budget_rows;
#[path = "runtime_row_data/foundation_rows.rs"]
mod foundation_rows;
#[path = "runtime_row_data/lock_poison_scene_script_rows.rs"]
mod lock_poison_scene_script_rows;
#[path = "runtime_row_data/status_support_priority_rows.rs"]
mod status_support_priority_rows;

// Parent-level mirror for source guards that read this route file directly.
// Runtime 15 M3 scene-script row-data root inventory child split.
// Status: runtime_15_scene_script_row_data_root_inventory_child_split_static_passed_cargo_deferred.
// Files:
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/root_paths.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/root_statuses.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/root_child_rows.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/root_owner_paths.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/root_inventory.rs
// Guard: runtime_15_scene_script_row_data_root_inventory_is_child_owned.
// Cargo gate deferred.

pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = lock_poison_scene_script_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = status_support_priority_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    asset_budget_rows::EXPECTED_STATUS_OUTPUT_SLICES;
