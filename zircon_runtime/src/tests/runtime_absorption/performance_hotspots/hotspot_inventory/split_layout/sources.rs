pub(super) struct HotspotInventorySplitSources {
    pub(super) parent: &'static str,
    pub(super) sources: &'static str,
    pub(super) evidence_gate_docs: &'static str,
    pub(super) ecs_extract_counters: &'static str,
    pub(super) ecs_extract_asset_animation: &'static str,
    pub(super) ecs_extract_extract_cache: &'static str,
    pub(super) ecs_extract_frame_diagnostics: &'static str,
    pub(super) ecs_extract_query_change: &'static str,
    pub(super) ecs_extract_split_layout: &'static str,
    pub(super) profiling_trace_render: &'static str,
    pub(super) split_layout_route: &'static str,
    pub(super) split_layout_sources: &'static str,
    pub(super) split_layout_route_child: &'static str,
    pub(super) split_layout_source_inventory: &'static str,
    pub(super) split_layout_status_docs: &'static str,
    pub(super) source_inventory: &'static str,
    pub(super) runtime_15_plan: &'static str,
    pub(super) runtime_index: &'static str,
    pub(super) runtime_07_plan: &'static str,
    pub(super) review_findings: &'static str,
    pub(super) structure_convention: &'static str,
    pub(super) module_doc: &'static str,
    pub(super) hotspot_doc: &'static str,
    pub(super) status_rows: &'static str,
    pub(super) status_slice: &'static str,
    pub(super) date_slice: &'static str,
    pub(super) session_note: &'static str,
}

pub(super) fn load() -> HotspotInventorySplitSources {
    HotspotInventorySplitSources {
        parent: include_str!("../../hotspot_inventory.rs"),
        sources: include_str!("../sources.rs"),
        evidence_gate_docs: include_str!("../evidence_gate_docs.rs"),
        ecs_extract_counters: include_str!("../ecs_extract_counters.rs"),
        ecs_extract_asset_animation: include_str!("../ecs_extract_counters/asset_animation.rs"),
        ecs_extract_extract_cache: include_str!("../ecs_extract_counters/extract_cache.rs"),
        ecs_extract_frame_diagnostics: include_str!("../ecs_extract_counters/frame_diagnostics.rs"),
        ecs_extract_query_change: include_str!("../ecs_extract_counters/query_change.rs"),
        ecs_extract_split_layout: include_str!("../ecs_extract_counters/split_layout.rs"),
        profiling_trace_render: include_str!("../profiling_trace_render.rs"),
        split_layout_route: include_str!("../split_layout.rs"),
        split_layout_sources: include_str!("sources.rs"),
        split_layout_route_child: include_str!("route.rs"),
        split_layout_source_inventory: include_str!("source_inventory.rs"),
        split_layout_status_docs: include_str!("status_docs.rs"),
        source_inventory: include_str!(
            "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
        ),
        runtime_15_plan: include_str!(
            "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
        ),
        runtime_index: include_str!(
            "../../../../../../../docs/plans/zircon_runtime/runtime/index.md"
        ),
        runtime_07_plan: include_str!(
            "../../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
        ),
        review_findings: include_str!(
            "../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
        ),
        structure_convention: include_str!(
            "../../../../../../../docs/plans/engine-code-structure-convention.md"
        ),
        module_doc: include_str!(
            "../../../../../../../docs/zircon_runtime/structure/module-convention.md"
        ),
        hotspot_doc: include_str!(
            "../../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md"
        ),
        status_rows: include_str!(
            "../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs"
        ),
        status_slice: include_str!(
            "../../../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps.rs"
        ),
        date_slice: include_str!(
            "../../../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/runtime07_script_maps.rs"
        ),
        session_note: include_str!(
            "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
        ),
    }
}

pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should retain hotspot-inventory split anchor `{anchor}`"
        );
    }
}
