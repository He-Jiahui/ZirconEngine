use super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_hotspot_inventory_routes(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "hotspot inventory child",
        sources.hotspot_inventory,
        &[
            "fn runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
            "#[path = \"hotspot_inventory/ecs_extract_counters.rs\"]",
            "#[path = \"hotspot_inventory/split_layout.rs\"]",
        ],
    );
    assert_contains_all(
        "hotspot inventory ECS/extract route",
        sources.hotspot_inventory_ecs_extract,
        &[
            "#[path = \"ecs_extract_counters/asset_animation.rs\"]",
            "#[path = \"ecs_extract_counters/extract_cache.rs\"]",
            "#[path = \"ecs_extract_counters/frame_diagnostics.rs\"]",
            "#[path = \"ecs_extract_counters/query_change.rs\"]",
            "#[path = \"ecs_extract_counters/split_layout.rs\"]",
        ],
    );
    assert_contains_all(
        "hotspot inventory ECS/extract support children",
        &format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            sources.hotspot_inventory_ecs_extract_asset_animation,
            sources.hotspot_inventory_ecs_extract_extract_cache,
            sources.hotspot_inventory_ecs_extract_frame_diagnostics,
            sources.hotspot_inventory_ecs_extract_query_change,
            sources.hotspot_inventory_ecs_extract_split_layout,
            sources.hotspot_inventory_ecs_extract_split_layout_route,
            sources.hotspot_inventory_ecs_extract_split_layout_source_inventory,
            sources.hotspot_inventory_ecs_extract_split_layout_sources,
            sources.hotspot_inventory_ecs_extract_split_layout_status_docs
        ),
        &[
            "assert_asset_and_animation_evidence",
            "assert_extract_evidence",
            "assert_ecs_frame_diagnostic_aggregation",
            "assert_query_and_change_evidence",
            "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split",
            "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_split",
            "hotspot_inventory/ecs_extract_counters/split_layout/source_inventory.rs",
            "assert_ecs_extract_counters_split_docs",
        ],
    );
}
