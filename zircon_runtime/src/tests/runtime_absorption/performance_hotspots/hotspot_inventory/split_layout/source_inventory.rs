use super::sources::{assert_contains_all, HotspotInventorySplitSources};

pub(super) fn assert_hotspot_inventory_source_inventory(sources: &HotspotInventorySplitSources) {
    assert_contains_all(
        "performance hotpath source inventory",
        sources.source_inventory,
        &[
            "RUNTIME_07_TEST_FILES = (",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/sources.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/evidence_gate_docs.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/query_change.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/extract_cache.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/asset_animation.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/frame_diagnostics.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/profiling_trace_render.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout/route.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout/source_inventory.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout/sources.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout/status_docs.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
        ],
    );
}
