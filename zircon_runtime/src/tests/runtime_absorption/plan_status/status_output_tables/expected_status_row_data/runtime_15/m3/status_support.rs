use super::ExpectedStatusOutputSlice;

#[path = "status_support/anchor_mirror.rs"]
mod anchor_mirror;
#[path = "status_support/expected_slice_maps.rs"]
mod expected_slice_maps;
#[path = "status_support/priority_plan_docs.rs"]
mod priority_plan_docs;
#[path = "status_support/row_data_and_budget.rs"]
mod row_data_and_budget;
#[path = "status_support/runtime_index_anchors.rs"]
mod runtime_index_anchors;

// Parent-level mirrors for source guards that read this route file directly.
// Runtime 15 M3 production file budget core runtime guard split.
// Status: runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred.
// Files:
// - structure_convention/production_file_budget.rs
// - structure_convention/production_file_budget/core_runtime_service_lists.rs
// Guard: runtime_15_production_file_budget_core_runtime_guard_is_child_owner.
// Cargo gate deferred.
// Runtime 15 M3 render shader template assembly guard support child-owner split.
// Status: runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred.
// Files:
// - structure_convention/production_file_budget/render_shader_template_assembly.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/sources.rs
// Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
// Cargo gate deferred.
// Status: runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred.
// Files:
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/template_contracts.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_cache_contracts.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs
// Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
// Cargo gate deferred.
// Runtime 15 M3 mesh pipeline shader source tests child-owner split.
// Status: runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred.
// Files:
// - graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
// - graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs
// - graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs
// Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
// Cargo gate deferred.

pub(super) const ROW_DATA_AND_BUDGET_TEST_FILE_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    row_data_and_budget::TEST_FILE_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_AND_BUDGET_RUNTIME_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    row_data_and_budget::RUNTIME_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_AND_BUDGET_ANCHOR_MIRROR_ROW_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    row_data_and_budget::ANCHOR_MIRROR_ROW_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_AND_BUDGET_HUB_EDITOR_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    row_data_and_budget::HUB_EDITOR_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_AND_BUDGET_RENDER_SHADER_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    row_data_and_budget::RENDER_SHADER_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_AND_BUDGET_M3_M4_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    row_data_and_budget::M3_M4_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_BASE_MAPS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = expected_slice_maps::BASE_MAPS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_TOP_LEVEL_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    expected_slice_maps::TOP_LEVEL_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_ROUTE_METADATA_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    expected_slice_maps::ROUTE_METADATA_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_STRUCTURE_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    expected_slice_maps::STRUCTURE_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_STATUS_SUPPORT_MAPS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    expected_slice_maps::STATUS_SUPPORT_MAPS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    expected_slice_maps::REVIEW_GUARD_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const EXPECTED_SLICE_WARNING_CLEANUP_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    expected_slice_maps::WARNING_CLEANUP_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_INDEX_ANCHORS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_index_anchors::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    priority_plan_docs::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PRIORITY_PLAN_DOCS_OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = priority_plan_docs::OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PRIORITY_PLAN_DOCS_OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    priority_plan_docs::OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PRIORITY_PLAN_DOCS_STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    priority_plan_docs::STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PRIORITY_PLAN_DOCS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = priority_plan_docs::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_SUPPORT_ROW_DATA_ANCHOR_MIRROR: &str =
    anchor_mirror::STATUS_SUPPORT_ROW_DATA_ANCHOR_MIRROR;
