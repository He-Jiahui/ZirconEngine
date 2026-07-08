#[path = "sources/load.rs"]
mod load_sources;

pub(super) struct OwnerBudgetSources {
    pub(super) performance_parent: &'static str,
    pub(super) artifact_render_diagnostics: &'static str,
    pub(super) artifact_render_diagnostics_artifact_cache_payload: &'static str,
    pub(super) artifact_render_diagnostics_render_product_diagnostics: &'static str,
    pub(super) artifact_render_diagnostics_split_layout: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_route: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_source_inventory: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_sources: &'static str,
    pub(super) artifact_render_diagnostics_split_layout_status_docs: &'static str,
    pub(super) hotspot_inventory: &'static str,
    pub(super) hotspot_inventory_ecs_extract: &'static str,
    pub(super) hotspot_inventory_ecs_extract_asset_animation: &'static str,
    pub(super) hotspot_inventory_ecs_extract_extract_cache: &'static str,
    pub(super) hotspot_inventory_ecs_extract_frame_diagnostics: &'static str,
    pub(super) hotspot_inventory_ecs_extract_query_change: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_route: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_source_inventory: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_sources: &'static str,
    pub(super) hotspot_inventory_ecs_extract_split_layout_status_docs: &'static str,
    pub(super) hotspot_inventory_evidence_gate: &'static str,
    pub(super) hotspot_inventory_profiling_trace: &'static str,
    pub(super) hotspot_inventory_sources: &'static str,
    pub(super) hotspot_inventory_split_layout: &'static str,
    pub(super) owner_budget: &'static str,
    pub(super) owner_budget_child_routes: &'static str,
    pub(super) owner_budget_child_routes_artifact_render_diagnostics: &'static str,
    pub(super) owner_budget_child_routes_hotspot_inventory: &'static str,
    pub(super) owner_budget_child_routes_owner_budget: &'static str,
    pub(super) owner_budget_child_routes_scene_project: &'static str,
    pub(super) owner_budget_child_routes_submit_context: &'static str,
    pub(super) owner_budget_large_file_gate: &'static str,
    pub(super) owner_budget_line_budgets: &'static str,
    pub(super) owner_budget_line_budgets_artifact_render_diagnostics: &'static str,
    pub(super) owner_budget_line_budgets_hotspot_inventory: &'static str,
    pub(super) owner_budget_line_budgets_owner_budget: &'static str,
    pub(super) owner_budget_line_budgets_root: &'static str,
    pub(super) owner_budget_line_budgets_scene_project: &'static str,
    pub(super) owner_budget_line_budgets_submit_context: &'static str,
    pub(super) owner_budget_mirror_docs: &'static str,
    pub(super) owner_budget_mirror_docs_audit_wiring: &'static str,
    pub(super) owner_budget_mirror_docs_doc_mirrors: &'static str,
    pub(super) owner_budget_mirror_docs_performance_guard: &'static str,
    pub(super) owner_budget_mirror_docs_source_inventory: &'static str,
    pub(super) owner_budget_mirror_docs_sources: &'static str,
    pub(super) owner_budget_mirror_docs_split_layout: &'static str,
    pub(super) owner_budget_parent_routes: &'static str,
    pub(super) owner_budget_source_inventory: &'static str,
    pub(super) owner_budget_sources: &'static str,
    pub(super) owner_budget_sources_load: &'static str,
    pub(super) owner_budget_split_layout: &'static str,
    pub(super) owner_budget_split_layout_route: &'static str,
    pub(super) owner_budget_split_layout_route_parent_route: &'static str,
    pub(super) owner_budget_split_layout_route_split_route: &'static str,
    pub(super) owner_budget_split_layout_route_support_routes: &'static str,
    pub(super) owner_budget_split_layout_source_inventory: &'static str,
    pub(super) owner_budget_split_layout_status_docs: &'static str,
    pub(super) owner_budget_status_docs: &'static str,
    pub(super) owner_budget_virtual_geometry_debug_snapshot: &'static str,
    pub(super) scene_project_splits: &'static str,
    pub(super) scene_project_splits_dynamic_session_event: &'static str,
    pub(super) scene_project_splits_project_io: &'static str,
    pub(super) scene_project_splits_scene_asset: &'static str,
    pub(super) scene_project_splits_split_layout: &'static str,
    pub(super) scene_project_splits_split_layout_route: &'static str,
    pub(super) scene_project_splits_split_layout_source_inventory: &'static str,
    pub(super) scene_project_splits_split_layout_sources: &'static str,
    pub(super) scene_project_splits_split_layout_status_docs: &'static str,
    pub(super) submit_context: &'static str,
    pub(super) submit_context_camera_loop: &'static str,
    pub(super) submit_context_feedback_sidebands: &'static str,
    pub(super) submit_context_source_extract_payloads: &'static str,
    pub(super) submit_context_sources: &'static str,
    pub(super) submit_context_split_layout: &'static str,
    pub(super) submit_context_split_layout_route: &'static str,
    pub(super) submit_context_split_layout_source_inventory: &'static str,
    pub(super) submit_context_split_layout_sources: &'static str,
    pub(super) submit_context_split_layout_status_docs: &'static str,
    pub(super) submit_context_status_docs: &'static str,
    pub(super) submit_error_paths: &'static str,
    pub(super) source_inventory: &'static str,
    pub(super) runtime_07_plan: &'static str,
    pub(super) runtime_15_plan: &'static str,
    pub(super) runtime_index: &'static str,
    pub(super) review_findings: &'static str,
    pub(super) structure_convention: &'static str,
    pub(super) module_doc: &'static str,
    pub(super) hotspot_doc: &'static str,
    pub(super) dynamic_session_doc: &'static str,
    pub(super) ecs_doc: &'static str,
    pub(super) interface_doc: &'static str,
    pub(super) architecture_review: &'static str,
    pub(super) status_rows: &'static str,
    pub(super) status_slice: &'static str,
    pub(super) date_slice: &'static str,
    pub(super) session_note: &'static str,
}

pub(super) fn load() -> OwnerBudgetSources {
    load_sources::load()
}

pub(super) fn assert_sources_guard_folder_backed(sources: &OwnerBudgetSources) {
    super::assert_contains_all(
        "owner-budget sources route",
        sources.owner_budget_sources,
        &[
            "#[path = \"sources/load.rs\"]",
            "mod load_sources;",
            "pub(super) struct OwnerBudgetSources",
            "pub(super) fn load() -> OwnerBudgetSources",
            "load_sources::load()",
            "pub(super) fn assert_sources_guard_folder_backed",
            "owner_budget_sources_load",
        ],
    );
    let source_loading_macro = ["include_", "str!("].concat();
    assert!(
        !sources.owner_budget_sources.contains(&source_loading_macro),
        "owner_budget/sources.rs should route instead of owning source-loading macros"
    );
    super::assert_contains_all(
        "owner-budget sources load child",
        sources.owner_budget_sources_load,
        &[
            "pub(super) fn load() -> OwnerBudgetSources",
            "performance_hotpath_source_inventory.py",
            "../../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md",
        ],
    );
    let source_load_child_anchor = [
        "owner_budget_sources_load: ",
        "include_",
        "str!(\"load.rs\")",
    ]
    .concat();
    assert!(
        sources
            .owner_budget_sources_load
            .contains(&source_load_child_anchor),
        "owner_budget/sources/load.rs should own the source-loading child include"
    );
}
