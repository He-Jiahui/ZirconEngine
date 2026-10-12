//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。集中保存路径、锚点或预期清单；消费方负责读取实际源码。
use super::sources::{assert_contains_all, HotspotInventorySplitSources};

pub(super) fn assert_hotspot_inventory_source_inventory(sources: &HotspotInventorySplitSources) {
    assert_contains_all(
        "performance hotpath source inventory",
        sources.source_inventory,
        &[
            "RUNTIME_07_TEST_FILES = (",
            "zircon_runtime/src/tests/contracts/performance/hotspot_inventory/sources.rs",
            "zircon_runtime/src/tests/contracts/hotspot_inventory/evidence_gate_docs.rs",
            "zircon_runtime/src/tests/contracts/hotspot_inventory/ecs_extract_counters.rs",
            "zircon_runtime/src/tests/contracts/ecs_extract_counters/query_change.rs",
            "zircon_runtime/src/tests/contracts/ecs_extract_counters/extract_cache.rs",
            "zircon_runtime/src/tests/contracts/ecs_extract_counters/asset_animation.rs",
            "zircon_runtime/src/tests/contracts/ecs_extract_counters/frame_diagnostics.rs",
            "zircon_runtime/src/tests/contracts/ecs_extract_counters/split_layout.rs",
            "zircon_runtime/src/tests/contracts/hotspot_inventory/profiling_trace_render.rs",
            "zircon_runtime/src/tests/contracts/performance/hotspot_inventory/split_layout.rs",
            "zircon_runtime/src/tests/contracts/split_layout/hotspot_inventory_route.rs",
            "zircon_runtime/src/tests/split_layout/hotspot_inventory_source_inventory.rs",
            "zircon_runtime/src/tests/contracts/split_layout/hotspot_inventory_sources.rs",
            "zircon_runtime/src/tests/contracts/split_layout/hotspot_inventory_status_docs.rs",
            "zircon_runtime/src/tests/contracts/performance/owner_budget/sources/load.rs",
        ],
    );
}
