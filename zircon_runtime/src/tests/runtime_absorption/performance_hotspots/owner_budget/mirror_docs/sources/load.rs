use super::MirrorDocsSources;

pub(super) fn load() -> MirrorDocsSources {
    MirrorDocsSources {
        runtime_07_archive: include_str!(
            "../../../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
        ),
        audit_script: include_str!(
            "../../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py"
        ),
        audit_source_inventory: include_str!(
            "../../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
        ),
        audit_anchor_inventory: include_str!(
            "../../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py"
        ),
        performance_guard: include_str!("../../../../performance_hotspots.rs"),
        artifact_render_diagnostics_guard: include_str!(
            "../../../artifact_render_diagnostics_splits.rs"
        ),
        artifact_render_diagnostics_artifact_cache_payload_guard: include_str!(
            "../../../artifact_render_diagnostics_splits/artifact_cache_payload.rs"
        ),
        artifact_render_diagnostics_render_product_diagnostics_guard: include_str!(
            "../../../artifact_render_diagnostics_splits/render_product_diagnostics.rs"
        ),
        artifact_render_diagnostics_split_layout_guard: include_str!(
            "../../../artifact_render_diagnostics_splits/split_layout.rs"
        ),
        artifact_render_diagnostics_split_layout_route_guard: include_str!(
            "../../../artifact_render_diagnostics_splits/split_layout/route.rs"
        ),
        artifact_render_diagnostics_split_layout_source_inventory_guard: include_str!(
            "../../../artifact_render_diagnostics_splits/split_layout/source_inventory.rs"
        ),
        artifact_render_diagnostics_split_layout_sources_guard: include_str!(
            "../../../artifact_render_diagnostics_splits/split_layout/sources.rs"
        ),
        artifact_render_diagnostics_split_layout_status_docs_guard: include_str!(
            "../../../artifact_render_diagnostics_splits/split_layout/status_docs.rs"
        ),
        hotspot_inventory_guard: include_str!("../../../hotspot_inventory.rs"),
        hotspot_inventory_ecs_extract_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters.rs"
        ),
        hotspot_inventory_ecs_extract_asset_animation_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/asset_animation.rs"
        ),
        hotspot_inventory_ecs_extract_extract_cache_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/extract_cache.rs"
        ),
        hotspot_inventory_ecs_extract_frame_diagnostics_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/frame_diagnostics.rs"
        ),
        hotspot_inventory_ecs_extract_query_change_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/query_change.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/split_layout.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_route_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/split_layout/route.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_source_inventory_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/split_layout/source_inventory.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_sources_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/split_layout/sources.rs"
        ),
        hotspot_inventory_ecs_extract_split_layout_status_docs_guard: include_str!(
            "../../../hotspot_inventory/ecs_extract_counters/split_layout/status_docs.rs"
        ),
        hotspot_inventory_evidence_gate_guard: include_str!(
            "../../../hotspot_inventory/evidence_gate_docs.rs"
        ),
        hotspot_inventory_profiling_trace_guard: include_str!(
            "../../../hotspot_inventory/profiling_trace_render.rs"
        ),
        hotspot_inventory_sources_guard: include_str!("../../../hotspot_inventory/sources.rs"),
        hotspot_inventory_split_layout_guard: include_str!(
            "../../../hotspot_inventory/split_layout.rs"
        ),
        owner_budget_guard: include_str!("../../../owner_budget.rs"),
        owner_budget_child_routes_guard: include_str!("../../child_routes.rs"),
        owner_budget_large_file_guard: include_str!("../../large_file_gate.rs"),
        owner_budget_line_budgets_guard: include_str!("../../line_budgets.rs"),
        owner_budget_mirror_docs_guard: include_str!("../../mirror_docs.rs"),
        owner_budget_mirror_docs_audit_wiring_guard: include_str!("../audit_wiring.rs"),
        owner_budget_mirror_docs_doc_mirrors_guard: include_str!("../doc_mirrors.rs"),
        owner_budget_mirror_docs_performance_guard: include_str!("../performance_guard.rs"),
        owner_budget_mirror_docs_source_inventory_guard: include_str!("../source_inventory.rs"),
        owner_budget_mirror_docs_sources_guard: include_str!("../sources.rs"),
        owner_budget_mirror_docs_sources_assertions_guard: include_str!("assertions.rs"),
        owner_budget_mirror_docs_sources_load_guard: include_str!("load.rs"),
        owner_budget_mirror_docs_sources_views_guard: include_str!("views.rs"),
        owner_budget_mirror_docs_split_layout_guard: include_str!("../split_layout.rs"),
        owner_budget_parent_routes_guard: include_str!("../../parent_routes.rs"),
        owner_budget_source_inventory_guard: include_str!("../../source_inventory.rs"),
        owner_budget_sources_guard: include_str!("../../sources.rs"),
        owner_budget_sources_load_guard: include_str!("../../sources/load.rs"),
        owner_budget_split_layout_guard: include_str!("../../split_layout.rs"),
        owner_budget_status_docs_guard: include_str!("../../status_docs.rs"),
        owner_budget_virtual_geometry_debug_snapshot_guard: include_str!(
            "../../virtual_geometry_debug_snapshot.rs"
        ),
        scene_project_splits_guard: include_str!("../../../scene_project_splits.rs"),
        scene_project_splits_dynamic_session_event_guard: include_str!(
            "../../../scene_project_splits/dynamic_session_event.rs"
        ),
        scene_project_splits_project_io_guard: include_str!(
            "../../../scene_project_splits/project_io.rs"
        ),
        scene_project_splits_scene_asset_guard: include_str!(
            "../../../scene_project_splits/scene_asset.rs"
        ),
        scene_project_splits_split_layout_guard: include_str!(
            "../../../scene_project_splits/split_layout.rs"
        ),
        scene_project_splits_split_layout_route_guard: include_str!(
            "../../../scene_project_splits/split_layout/route.rs"
        ),
        scene_project_splits_split_layout_source_inventory_guard: include_str!(
            "../../../scene_project_splits/split_layout/source_inventory.rs"
        ),
        scene_project_splits_split_layout_sources_guard: include_str!(
            "../../../scene_project_splits/split_layout/sources.rs"
        ),
        scene_project_splits_split_layout_status_docs_guard: include_str!(
            "../../../scene_project_splits/split_layout/status_docs.rs"
        ),
        submit_context_guard: include_str!("../../../submit_context.rs"),
        submit_context_camera_loop_guard: include_str!(
            "../../../submit_context/camera_loop_sharing.rs"
        ),
        submit_context_feedback_sidebands_guard: include_str!(
            "../../../submit_context/feedback_sidebands.rs"
        ),
        submit_context_source_extract_payloads_guard: include_str!(
            "../../../submit_context/source_extract_payloads.rs"
        ),
        submit_context_sources_guard: include_str!("../../../submit_context/sources.rs"),
        submit_context_split_layout_guard: include_str!("../../../submit_context/split_layout.rs"),
        submit_context_status_docs_guard: include_str!("../../../submit_context/status_docs.rs"),
        submit_error_paths_guard: include_str!("../../../submit_error_paths.rs"),
    }
}
