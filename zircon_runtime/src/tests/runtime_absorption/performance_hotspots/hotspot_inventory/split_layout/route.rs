use super::sources::{assert_contains_all, HotspotInventorySplitSources};

pub(super) fn assert_hotspot_inventory_split_route(sources: &HotspotInventorySplitSources) {
    assert_contains_all(
        "hotspot inventory route",
        sources.parent,
        &[
            "#[path = \"hotspot_inventory/ecs_extract_counters.rs\"]",
            "#[path = \"hotspot_inventory/evidence_gate_docs.rs\"]",
            "#[path = \"hotspot_inventory/profiling_trace_render.rs\"]",
            "#[path = \"hotspot_inventory/sources.rs\"]",
            "#[path = \"hotspot_inventory/split_layout.rs\"]",
            "fn runtime_07_hotspot_inventory_requires_counted_evidence_before_m2()",
            "evidence_gate_docs::assert_evidence_gate_docs(&sources);",
            "ecs_extract_counters::assert_ecs_extract_counter_evidence(&sources);",
            "profiling_trace_render::assert_profiling_trace_and_render_diversion(&sources);",
        ],
    );

    for moved_anchor in [
        "for required_plan_anchor in [",
        "for required_query_anchor in [",
        "for required_profiling_build_anchor in [",
        "for required_render_anchor in [",
    ] {
        assert!(
            !sources.parent.contains(moved_anchor),
            "hotspot_inventory.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "hotspot inventory split-layout route",
        sources.split_layout_route,
        &[
            "#[path = \"split_layout/route.rs\"]",
            "#[path = \"split_layout/source_inventory.rs\"]",
            "#[path = \"split_layout/sources.rs\"]",
            "#[path = \"split_layout/status_docs.rs\"]",
            "route::assert_hotspot_inventory_split_route(&sources);",
            "source_inventory::assert_hotspot_inventory_source_inventory(&sources);",
            "status_docs::assert_hotspot_inventory_status_docs(&sources);",
        ],
    );

    for moved_anchor in [
        "let parent = include_str!",
        "let runtime_15_plan = include_str!",
        "for (path, source) in [",
        "for (label, source) in [",
    ] {
        assert!(
            !sources.split_layout_route.contains(moved_anchor),
            "hotspot_inventory/split_layout.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_hotspot_inventory_children(sources);
    assert_line_budgets(sources);
}

fn assert_hotspot_inventory_children(sources: &HotspotInventorySplitSources) {
    assert_contains_all(
        "hotspot inventory sources",
        sources.sources,
        &[
            "pub(super) struct HotspotInventorySources",
            "pub(super) fn load() -> Self",
            "07-runtime-performance-hotpath.md",
            "ecs_performance_acceptance.rs",
            "zircon_build.py",
        ],
    );
    assert_contains_all(
        "evidence gate docs child",
        sources.evidence_gate_docs,
        &[
            "assert_evidence_gate_docs",
            "No Runtime 07 M2 optimization slice may start from an unmeasured suspicion",
            "CounterHotspotReport",
        ],
    );
    assert_contains_all(
        "ECS/extract counters child",
        sources.ecs_extract_counters,
        &[
            "assert_ecs_extract_counter_evidence",
            "#[path = \"ecs_extract_counters/query_change.rs\"]",
            "#[path = \"ecs_extract_counters/split_layout.rs\"]",
            "query_change::assert_query_and_change_evidence(sources);",
        ],
    );
    assert_contains_all(
        "ECS/extract counter children",
        &format!(
            "{}\n{}\n{}\n{}\n{}",
            sources.ecs_extract_asset_animation,
            sources.ecs_extract_extract_cache,
            sources.ecs_extract_frame_diagnostics,
            sources.ecs_extract_query_change,
            sources.ecs_extract_split_layout
        ),
        &[
            "query_state_reuses_archetype_matches_across_unchanged_frames",
            "RuntimeFrameExtractCache",
            "AnimationSceneFrameDiagnostics",
            "EcsFramePerformanceDiagnostics",
            "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split",
        ],
    );
    assert_contains_all(
        "profiling trace/render child",
        sources.profiling_trace_render,
        &[
            "assert_profiling_trace_and_render_diversion",
            "profiling_build_tooling_static_passed_cargo_deferred_active_lanes",
            "direct_runtime_frame_submit_exports_perfetto_trace_artifacts",
            "Runtime 07 M2 is not allowed to fix render submission",
        ],
    );
}

fn assert_line_budgets(sources: &HotspotInventorySplitSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            sources.parent,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/sources.rs",
            sources.sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/evidence_gate_docs.rs",
            sources.evidence_gate_docs,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters.rs",
            sources.ecs_extract_counters,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/profiling_trace_render.rs",
            sources.profiling_trace_render,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout.rs",
            sources.split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout/route.rs",
            sources.split_layout_route_child,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout/sources.rs",
            sources.split_layout_sources,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 280,
            "{path} should stay below the focused hotspot-inventory guard budget; got {line_count} lines"
        );
    }
}
