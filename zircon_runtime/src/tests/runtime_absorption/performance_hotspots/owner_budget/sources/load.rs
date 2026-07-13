use super::OwnerBudgetSources;

pub(super) fn load() -> OwnerBudgetSources {
    OwnerBudgetSources {
        performance_parent: include_str!("../../../performance_hotspots.rs"),
        artifact_render_diagnostics: include_str!("../../artifact_render_diagnostics_splits.rs"),
        artifact_render_diagnostics_artifact_cache_payload: include_str!(
            "../../artifact_render_diagnostics_splits/artifact_cache_payload.rs"
        ),
        artifact_render_diagnostics_render_product_diagnostics: include_str!(
            "../../artifact_render_diagnostics_splits/render_product_diagnostics.rs"
        ),
        artifact_render_diagnostics_split_layout: include_str!(
            "../../artifact_render_diagnostics_splits/split_layout.rs"
        ),
        artifact_render_diagnostics_split_layout_route: include_str!(
            "../../artifact_render_diagnostics_splits/split_layout/route.rs"
        ),
        artifact_render_diagnostics_split_layout_source_inventory: include_str!(
            "../../artifact_render_diagnostics_splits/split_layout/source_inventory.rs"
        ),
        artifact_render_diagnostics_split_layout_sources: include_str!(
            "../../artifact_render_diagnostics_splits/split_layout/sources.rs"
        ),
        artifact_render_diagnostics_split_layout_status_docs: include_str!(
            "../../artifact_render_diagnostics_splits/split_layout/status_docs.rs"
        ),
        hotspot_inventory: include_str!("../../hotspot_inventory.rs"),
        hotspot_inventory_ecs_extract: include_str!(
            "../../hotspot_inventory/ecs_extract_counters.rs"
        ),
        hotspot_inventory_ecs_extract_asset_animation: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/asset_animation.rs"
        ),
        hotspot_inventory_ecs_extract_extract_cache: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/extract_cache.rs"
        ),
        hotspot_inventory_ecs_extract_frame_diagnostics: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/frame_diagnostics.rs"
        ),
        hotspot_inventory_ecs_extract_query_change: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/query_change.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/split_layout.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_route: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/split_layout/route.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_source_inventory: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/split_layout/source_inventory.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_sources: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/split_layout/sources.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_status_docs: include_str!(
            "../../hotspot_inventory/ecs_extract_counters/split_layout/status_docs.rs"
        ),
        hotspot_inventory_evidence_gate: include_str!(
            "../../hotspot_inventory/evidence_gate_docs.rs"
        ),
        hotspot_inventory_profiling_trace: include_str!(
            "../../hotspot_inventory/profiling_trace_render.rs"
        ),
        hotspot_inventory_sources: include_str!("../../hotspot_inventory/sources.rs"),
        hotspot_inventory_split_layout: include_str!("../../hotspot_inventory/split_layout.rs"),
        owner_budget: include_str!("../../owner_budget.rs"),
        owner_budget_child_routes: include_str!("../child_routes.rs"),
        owner_budget_child_routes_artifact_render_diagnostics: include_str!(
            "../child_routes/artifact_render_diagnostics.rs"
        ),
        owner_budget_child_routes_hotspot_inventory: include_str!(
            "../child_routes/hotspot_inventory.rs"
        ),
        owner_budget_child_routes_owner_budget: include_str!("../child_routes/owner_budget.rs"),
        owner_budget_child_routes_scene_project: include_str!("../child_routes/scene_project.rs"),
        owner_budget_child_routes_submit_context: include_str!("../child_routes/submit_context.rs"),
        owner_budget_large_file_gate: include_str!("../large_file_gate.rs"),
        owner_budget_line_budgets: include_str!("../line_budgets.rs"),
        owner_budget_line_budgets_artifact_render_diagnostics: include_str!(
            "../line_budgets/artifact_render_diagnostics.rs"
        ),
        owner_budget_line_budgets_hotspot_inventory: include_str!(
            "../line_budgets/hotspot_inventory.rs"
        ),
        owner_budget_line_budgets_owner_budget: include_str!(
            "../line_budgets/owner_budget.rs"
        ),
        owner_budget_line_budgets_root: include_str!("../line_budgets/root.rs"),
        owner_budget_line_budgets_scene_project: include_str!(
            "../line_budgets/scene_project.rs"
        ),
        owner_budget_line_budgets_submit_context: include_str!(
            "../line_budgets/submit_context.rs"
        ),
        owner_budget_mirror_docs: include_str!("../mirror_docs.rs"),
        owner_budget_mirror_docs_audit_wiring: include_str!("../mirror_docs/audit_wiring.rs"),
        owner_budget_mirror_docs_doc_mirrors: include_str!("../mirror_docs/doc_mirrors.rs"),
        owner_budget_mirror_docs_performance_guard: include_str!(
            "../mirror_docs/performance_guard.rs"
        ),
        owner_budget_mirror_docs_source_inventory: include_str!(
            "../mirror_docs/source_inventory.rs"
        ),
        owner_budget_mirror_docs_sources: include_str!("../mirror_docs/sources.rs"),
        owner_budget_mirror_docs_split_layout: include_str!("../mirror_docs/split_layout.rs"),
        owner_budget_parent_routes: include_str!("../parent_routes.rs"),
        owner_budget_source_inventory: include_str!("../source_inventory.rs"),
        owner_budget_sources: include_str!("../sources.rs"),
        owner_budget_sources_load: include_str!("load.rs"),
        owner_budget_split_layout: include_str!("../split_layout.rs"),
        owner_budget_split_layout_route: include_str!("../split_layout/route.rs"),
        owner_budget_split_layout_route_parent_route: include_str!(
            "../split_layout/route/parent_route.rs"
        ),
        owner_budget_split_layout_route_split_route: include_str!(
            "../split_layout/route/split_route.rs"
        ),
        owner_budget_split_layout_route_support_routes: include_str!(
            "../split_layout/route/support_routes.rs"
        ),
        owner_budget_split_layout_source_inventory: include_str!(
            "../split_layout/source_inventory.rs"
        ),
        owner_budget_split_layout_status_docs: include_str!("../split_layout/status_docs.rs"),
        owner_budget_status_docs: include_str!("../status_docs.rs"),
        owner_budget_virtual_geometry_debug_snapshot: include_str!(
            "../virtual_geometry_debug_snapshot.rs"
        ),
        scene_project_splits: include_str!("../../scene_project_splits.rs"),
        scene_project_splits_dynamic_session_event: include_str!(
            "../../scene_project_splits/dynamic_session_event.rs"
        ),
        scene_project_splits_project_io: include_str!("../../scene_project_splits/project_io.rs"),
        scene_project_splits_scene_asset: include_str!(
            "../../scene_project_splits/scene_asset.rs"
        ),
        scene_project_splits_split_layout: include_str!(
            "../../scene_project_splits/split_layout.rs"
        ),
        scene_project_splits_split_layout_route: include_str!(
            "../../scene_project_splits/split_layout/route.rs"
        ),
        scene_project_splits_split_layout_source_inventory: include_str!(
            "../../scene_project_splits/split_layout/source_inventory.rs"
        ),
        scene_project_splits_split_layout_sources: include_str!(
            "../../scene_project_splits/split_layout/sources.rs"
        ),
        scene_project_splits_split_layout_status_docs: include_str!(
            "../../scene_project_splits/split_layout/status_docs.rs"
        ),
        submit_context: include_str!("../../submit_context.rs"),
        submit_context_camera_loop: include_str!("../../submit_context/camera_loop_sharing.rs"),
        submit_context_feedback_sidebands: include_str!(
            "../../submit_context/feedback_sidebands.rs"
        ),
        submit_context_source_extract_payloads: include_str!(
            "../../submit_context/source_extract_payloads.rs"
        ),
        submit_context_sources: include_str!("../../submit_context/sources.rs"),
        submit_context_split_layout: include_str!("../../submit_context/split_layout.rs"),
        submit_context_split_layout_route: include_str!(
            "../../submit_context/split_layout/route.rs"
        ),
        submit_context_split_layout_source_inventory: include_str!(
            "../../submit_context/split_layout/source_inventory.rs"
        ),
        submit_context_split_layout_sources: include_str!(
            "../../submit_context/split_layout/sources.rs"
        ),
        submit_context_split_layout_status_docs: include_str!(
            "../../submit_context/split_layout/status_docs.rs"
        ),
        submit_context_status_docs: include_str!("../../submit_context/status_docs.rs"),
        submit_error_paths: include_str!("../../submit_error_paths.rs"),
        source_inventory: include_str!(
            "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
        ),
        runtime_07_archive: include_str!(
            "../../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
        ),
        runtime_15_archive: include_str!(
            "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
        ),
    }
}
