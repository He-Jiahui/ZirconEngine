use super::ExpectedStatusOutputSlice;

#[path = "production_guard_support/core_and_evidence.rs"]
mod core_and_evidence;
#[path = "production_guard_support/expected_slice_guards.rs"]
mod expected_slice_guards;
#[path = "production_guard_support/module_layout.rs"]
mod module_layout;
#[path = "production_guard_support/review_guard.rs"]
mod review_guard;
#[path = "production_guard_support/runtime_row_data.rs"]
mod runtime_row_data;
#[path = "production_guard_support/status_docs.rs"]
mod status_docs;

// Parent-level mirrors for source guards that read this route file directly.
// Runtime 15 M3 scene-script row-data guard folder-backed split.
// Status: runtime_15_scene_script_row_data_guard_folder_backed_static_passed_cargo_deferred.
// Files:
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/delegation.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/row_ownership.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/export_chain.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/status_mirrors.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/budgets.rs
// Guards:
// - runtime_15_scene_script_row_data_guard_is_folder_backed
// - runtime_15_scene_script_row_data_owner_is_child_backed
// Cargo gate deferred.
// Runtime 15 M3 scene-script row-data status-mirror child split.
// Status: runtime_15_scene_script_row_data_status_mirror_child_split_static_passed_cargo_deferred.
// Files:
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/status_mirrors.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/status_mirrors/child_split_status.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/status_mirrors/historical_status.rs
// - structure_convention/test_file_budget/status_output_row_data/runtime_15_scene_script_row_data/status_mirrors/folder_backed_status.rs
// Guard: runtime_15_scene_script_row_data_status_mirror_children_are_child_owned.
// Cargo gate deferred.
// Runtime 15 M3 production file budget guard child-owner split.
// Status: runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred.
// Files:
// - structure_convention/production_file_budget.rs
// - structure_convention/production_file_budget/module_layout.rs
// Guard: runtime_15_production_file_budget_guard_child_owner_split.
// Cargo gate deferred.

pub(super) const CORE_AND_EVIDENCE_PRODUCTION_FILE_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    core_and_evidence::PRODUCTION_FILE_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CORE_AND_EVIDENCE_EVIDENCE_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = core_and_evidence::EVIDENCE_ANCHOR_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CORE_AND_EVIDENCE_CHILD_GROUP_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    core_and_evidence::CHILD_GROUP_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_ROOT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    core_and_evidence::CHILD_GROUP_INVENTORY_ROOT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_OWNER_PATH_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    core_and_evidence::CHILD_GROUP_INVENTORY_OWNER_PATH_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_ROOT_PATH_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    core_and_evidence::CHILD_GROUP_INVENTORY_ROOT_PATH_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_GUARD_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    core_and_evidence::CHILD_GROUP_INVENTORY_GUARD_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const MODULE_LAYOUT_BASE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    module_layout::BASE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const MODULE_LAYOUT_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = module_layout::STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const MODULE_LAYOUT_CHILD_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = module_layout::CHILD_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const MODULE_LAYOUT_CHILD_SUMMARY_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    module_layout::CHILD_SUMMARY_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_BASE_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    review_guard::BASE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_MOVED_ROW_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = review_guard::MOVED_ROW_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = review_guard::CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = review_guard::ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = review_guard::STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = review_guard::DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_row_data::FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_row_data::LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_row_data::STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_row_data::STATUS_SUPPORT_PRIORITY_ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_row_data::STATUS_SUPPORT_PRIORITY_PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    runtime_row_data::STATUS_SUPPORT_PRIORITY_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_ROW_DATA_ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_row_data::ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOCS_FOUNDATION_M2_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = status_docs::FOUNDATION_M2_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOCS_CHILD_GROUP_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    status_docs::CHILD_GROUP_STATUS_DOC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOCS_CHILD_GROUP_STATUS_ROW_DOC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    status_docs::CHILD_GROUP_STATUS_ROW_DOC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_DOCS_CHILD_GROUP_MOVED_ROW_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = status_docs::CHILD_GROUP_MOVED_ROW_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = expected_slice_guards::EXPECTED_STATUS_OUTPUT_SLICES;
