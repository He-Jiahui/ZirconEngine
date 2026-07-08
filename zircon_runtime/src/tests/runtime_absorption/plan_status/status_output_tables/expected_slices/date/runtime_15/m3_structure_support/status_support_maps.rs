#[path = "status_support_maps/evidence_maps.rs"]
mod evidence_maps;
#[path = "status_support_maps/foundation_row_data_maps.rs"]
mod foundation_row_data_maps;
#[path = "status_support_maps/hub_editor_maps.rs"]
mod hub_editor_maps;
#[path = "status_support_maps/m2_row_data_maps.rs"]
mod m2_row_data_maps;
#[path = "status_support_maps/m3_m4_expected_slice_maps.rs"]
mod m3_m4_expected_slice_maps;
#[path = "status_support_maps/plan_doc_support_maps.rs"]
mod plan_doc_support_maps;
#[path = "status_support_maps/render_shader_maps.rs"]
mod render_shader_maps;
#[path = "status_support_maps/root_layout_ui_maps.rs"]
mod root_layout_ui_maps;
#[path = "status_support_maps/row_data_maps.rs"]
mod row_data_maps;
#[path = "status_support_maps/runtime_row_data_maps.rs"]
mod runtime_row_data_maps;

// Parent-level mirror for source guards that read this route file directly.
// Runtime 15 M3 render shader template assembly guard support child-owner split.
// Status: runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred.
// Date: Some("2026-06-27").
// Files:
// - structure_convention/production_file_budget/render_shader_template_assembly.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/sources.rs
// Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
// Runtime 15 M3 render shader template assembly assertion contract child-owner split.
// Status: runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred.
// Date: Some("2026-07-01").
// Files:
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/template_contracts.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_cache_contracts.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs
// - structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs
// Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
// Runtime 15 M3 mesh pipeline shader source tests child-owner split.
// Status: runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred.
// Date: Some("2026-07-01").
// Files:
// - graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
// - graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs
// - graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs
// Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.

const REVIEW_GUARD_ROW_DATA_FOLDER_BACKED_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard row-data guard folder-backed split";
const REVIEW_GUARD_ROW_DATA_FOLDER_BACKED_DATE: &str = "2026-07-02";
const REVIEW_GUARD_ROW_DATA_STATUS_MIRROR_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard row-data status-mirror child split";
const REVIEW_GUARD_ROW_DATA_STATUS_MIRROR_DATE: &str = "2026-07-04";

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == REVIEW_GUARD_ROW_DATA_FOLDER_BACKED_STATUS_NAME {
        return Some(REVIEW_GUARD_ROW_DATA_FOLDER_BACKED_DATE);
    }
    if slice == REVIEW_GUARD_ROW_DATA_STATUS_MIRROR_STATUS_NAME {
        return Some(REVIEW_GUARD_ROW_DATA_STATUS_MIRROR_DATE);
    }
    row_data_maps::expected_date_for_slice(slice)
        .or_else(|| plan_doc_support_maps::expected_date_for_slice(slice))
        .or_else(|| runtime_row_data_maps::expected_date_for_slice(slice))
        .or_else(|| foundation_row_data_maps::expected_date_for_slice(slice))
        .or_else(|| m2_row_data_maps::expected_date_for_slice(slice))
        .or_else(|| hub_editor_maps::expected_date_for_slice(slice))
        .or_else(|| render_shader_maps::expected_date_for_slice(slice))
        .or_else(|| m3_m4_expected_slice_maps::expected_date_for_slice(slice))
        .or_else(|| root_layout_ui_maps::expected_date_for_slice(slice))
        .or_else(|| evidence_maps::expected_date_for_slice(slice))
}
