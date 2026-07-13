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
    pub(super) source_inventory: &'static str,
    pub(super) runtime_07_archive: &'static str,
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
        source_inventory: include_str!(
            "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
        ),
        runtime_07_archive: include_str!(
            "../../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
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
