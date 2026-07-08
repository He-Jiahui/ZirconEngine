use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_ecs_extract_counters_split_layout(sources: &SplitLayoutSources) {
    assert_ecs_extract_counters_parent_route(sources);
    assert_ecs_extract_counters_support_children(sources);
    assert_ecs_extract_counters_split_route(sources);
    assert_ecs_extract_counters_split_budgets(sources);
}

fn assert_ecs_extract_counters_parent_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "ecs_extract_counters route",
        sources.parent,
        &[
            "#[path = \"ecs_extract_counters/asset_animation.rs\"]",
            "#[path = \"ecs_extract_counters/extract_cache.rs\"]",
            "#[path = \"ecs_extract_counters/frame_diagnostics.rs\"]",
            "#[path = \"ecs_extract_counters/query_change.rs\"]",
            "#[path = \"ecs_extract_counters/split_layout.rs\"]",
            "query_change::assert_query_and_change_evidence(sources);",
            "extract_cache::assert_extract_evidence(sources);",
            "asset_animation::assert_asset_and_animation_evidence(sources);",
            "frame_diagnostics::assert_ecs_frame_diagnostic_aggregation(sources);",
        ],
    );

    for moved_anchor in [
        "for required_query_anchor in [",
        "for required_extract_anchor in [",
        "for required_asset_worker_anchor in [",
        "for required_ecs_frame_diagnostic_anchor in [",
    ] {
        assert!(
            !sources.parent.contains(moved_anchor),
            "ecs_extract_counters.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }
}

fn assert_ecs_extract_counters_support_children(sources: &SplitLayoutSources) {
    assert_contains_all(
        "query/change counter child",
        sources.query_change,
        &[
            "assert_query_and_change_evidence",
            "query_state_reuses_archetype_matches_across_unchanged_frames",
            "change_detection_scan_skips_unmarked_archetypes",
        ],
    );
    assert_contains_all(
        "extract/cache child",
        sources.extract_cache,
        &[
            "assert_extract_evidence",
            "frame_extract_rebuild_skips_unchanged_entities",
            "RuntimeFrameExtractCache",
        ],
    );
    assert_contains_all(
        "asset/animation counter child",
        sources.asset_animation,
        &[
            "assert_asset_and_animation_evidence",
            "AssetWorkerPoolFrameSampler",
            "AnimationSceneFrameDiagnostics",
        ],
    );
    assert_contains_all(
        "ECS frame diagnostics child",
        sources.frame_diagnostics,
        &[
            "assert_ecs_frame_diagnostic_aggregation",
            "EcsFramePerformanceDiagnostics",
            "record_ecs_query_cache_stats",
        ],
    );
}

fn assert_ecs_extract_counters_split_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "ecs_extract_counters split-layout route",
        sources.split_layout,
        &[
            "#[path = \"split_layout/route.rs\"]",
            "#[path = \"split_layout/source_inventory.rs\"]",
            "#[path = \"split_layout/sources.rs\"]",
            "#[path = \"split_layout/status_docs.rs\"]",
            "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split",
            "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_split",
            "run_ecs_extract_counters_split_layout_checks();",
            "route::assert_ecs_extract_counters_split_layout(&sources);",
            "source_inventory::assert_ecs_extract_counters_source_inventory(&sources);",
            "status_docs::assert_ecs_extract_counters_split_docs(&sources);",
        ],
    );

    for moved_anchor in [
        "let parent = include_str!(\"../ecs_extract_counters.rs\")",
        "let source_inventory = include_str!",
        "for moved_anchor in [",
        "for (path, source) in [",
        "for (label, source) in [",
    ] {
        assert!(
            !sources.split_layout.contains(moved_anchor),
            "ecs_extract_counters/split_layout.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "ecs_extract_counters split-layout children",
        &format!(
            "{}\n{}\n{}\n{}",
            sources.split_layout_route,
            sources.split_layout_source_inventory,
            sources.split_layout_sources,
            sources.split_layout_status_docs
        ),
        &[
            "assert_ecs_extract_counters_split_layout",
            "assert_ecs_extract_counters_source_inventory",
            "pub(super) struct SplitLayoutSources",
            "assert_ecs_extract_counters_split_docs",
            "Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters split-layout guard folder-backed split",
        ],
    );
}

fn assert_ecs_extract_counters_split_budgets(sources: &SplitLayoutSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters.rs",
            sources.parent,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/query_change.rs",
            sources.query_change,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/extract_cache.rs",
            sources.extract_cache,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/asset_animation.rs",
            sources.asset_animation,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/frame_diagnostics.rs",
            sources.frame_diagnostics,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout.rs",
            sources.split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/route.rs",
            sources.split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/source_inventory.rs",
            sources.split_layout_source_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/sources.rs",
            sources.split_layout_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/status_docs.rs",
            sources.split_layout_status_docs,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 260,
            "{path} should stay below the focused ECS/extract counter split-layout guard budget; got {line_count} lines"
        );
    }
}
