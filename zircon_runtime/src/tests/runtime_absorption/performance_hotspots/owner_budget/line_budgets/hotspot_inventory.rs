use super::super::sources::OwnerBudgetSources;

pub(super) fn assert_hotspot_inventory_budgets(sources: &OwnerBudgetSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            sources.hotspot_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters.rs",
            sources.hotspot_inventory_ecs_extract,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/asset_animation.rs",
            sources.hotspot_inventory_ecs_extract_asset_animation,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/extract_cache.rs",
            sources.hotspot_inventory_ecs_extract_extract_cache,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/frame_diagnostics.rs",
            sources.hotspot_inventory_ecs_extract_frame_diagnostics,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/query_change.rs",
            sources.hotspot_inventory_ecs_extract_query_change,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout.rs",
            sources.hotspot_inventory_ecs_extract_split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/route.rs",
            sources.hotspot_inventory_ecs_extract_split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/source_inventory.rs",
            sources.hotspot_inventory_ecs_extract_split_layout_source_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/sources.rs",
            sources.hotspot_inventory_ecs_extract_split_layout_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/status_docs.rs",
            sources.hotspot_inventory_ecs_extract_split_layout_status_docs,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/evidence_gate_docs.rs",
            sources.hotspot_inventory_evidence_gate,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/profiling_trace_render.rs",
            sources.hotspot_inventory_profiling_trace,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/sources.rs",
            sources.hotspot_inventory_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout.rs",
            sources.hotspot_inventory_split_layout,
        ),
    ] {
        super::assert_runtime_15_test_file_budget(path, source);
    }
}
