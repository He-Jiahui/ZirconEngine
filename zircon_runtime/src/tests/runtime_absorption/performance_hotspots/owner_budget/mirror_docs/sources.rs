#[path = "sources/assertions.rs"]
mod assertions;
#[path = "sources/load.rs"]
mod load_sources;
#[path = "sources/views.rs"]
mod views;

pub(super) struct MirrorDocsSources {
    pub(super) runtime_07_archive: &'static str,
    pub(super) audit_script: &'static str,
    pub(super) audit_source_inventory: &'static str,
    pub(super) audit_anchor_inventory: &'static str,
    pub(super) performance_guard: &'static str,
    pub(super) artifact_render_diagnostics_guard: &'static str,
    pub(super) artifact_render_diagnostics_artifact_cache_payload_guard: &'static str,
    pub(super) artifact_render_diagnostics_render_product_diagnostics_guard: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_guard: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_route_guard: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_source_inventory_guard: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_sources_guard: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_status_docs_guard: &'static str,
    pub(super) hotspot_inventory_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_asset_animation_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_extract_cache_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_frame_diagnostics_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_query_change_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_route_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_source_inventory_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_sources_guard: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_status_docs_guard: &'static str,
    pub(super) hotspot_inventory_evidence_gate_guard: &'static str,
    pub(super) hotspot_inventory_profiling_trace_guard: &'static str,
    pub(super) hotspot_inventory_sources_guard: &'static str,
    pub(super) hotspot_inventory_split_layout_guard: &'static str,
    pub(super) owner_budget_guard: &'static str,
    pub(super) owner_budget_child_routes_guard: &'static str,
    pub(super) owner_budget_large_file_guard: &'static str,
    pub(super) owner_budget_line_budgets_guard: &'static str,
    pub(super) owner_budget_mirror_docs_guard: &'static str,
    pub(super) owner_budget_mirror_docs_audit_wiring_guard: &'static str,
    pub(super) owner_budget_mirror_docs_doc_mirrors_guard: &'static str,
    pub(super) owner_budget_mirror_docs_performance_guard: &'static str,
    pub(super) owner_budget_mirror_docs_source_inventory_guard: &'static str,
    pub(super) owner_budget_mirror_docs_sources_guard: &'static str,
    pub(super) owner_budget_mirror_docs_sources_assertions_guard: &'static str,
    pub(super) owner_budget_mirror_docs_sources_load_guard: &'static str,
    pub(super) owner_budget_mirror_docs_sources_views_guard: &'static str,
    pub(super) owner_budget_mirror_docs_split_layout_guard: &'static str,
    pub(super) owner_budget_parent_routes_guard: &'static str,
    pub(super) owner_budget_source_inventory_guard: &'static str,
    pub(super) owner_budget_sources_guard: &'static str,
    pub(super) owner_budget_sources_load_guard: &'static str,
    pub(super) owner_budget_split_layout_guard: &'static str,
    pub(super) owner_budget_status_docs_guard: &'static str,
    pub(super) owner_budget_virtual_geometry_debug_snapshot_guard: &'static str,
    pub(super) scene_project_splits_guard: &'static str,
    pub(super) scene_project_splits_dynamic_session_event_guard: &'static str,
    pub(super) scene_project_splits_project_io_guard: &'static str,
    pub(super) scene_project_splits_scene_asset_guard: &'static str,
    pub(super) scene_project_splits_split_layout_guard: &'static str,
    pub(super) scene_project_splits_split_layout_route_guard: &'static str,
    pub(super) scene_project_splits_split_layout_source_inventory_guard: &'static str,
    pub(super) scene_project_splits_split_layout_sources_guard: &'static str,
    pub(super) scene_project_splits_split_layout_status_docs_guard: &'static str,
    pub(super) submit_context_guard: &'static str,
    pub(super) submit_context_camera_loop_guard: &'static str,
    pub(super) submit_context_feedback_sidebands_guard: &'static str,
    pub(super) submit_context_source_extract_payloads_guard: &'static str,
    pub(super) submit_context_sources_guard: &'static str,
    pub(super) submit_context_split_layout_guard: &'static str,
    pub(super) submit_context_status_docs_guard: &'static str,
    pub(super) submit_error_paths_guard: &'static str,
}

pub(super) fn load() -> MirrorDocsSources {
    load_sources::load()
}

pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    assertions::assert_contains_all(label, source, anchors);
}
