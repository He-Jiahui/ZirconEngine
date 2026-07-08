use super::super::sources::OwnerBudgetSources;

pub(super) fn assert_scene_project_budgets(sources: &OwnerBudgetSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits.rs",
            sources.scene_project_splits,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs",
            sources.scene_project_splits_dynamic_session_event,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/project_io.rs",
            sources.scene_project_splits_project_io,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/scene_asset.rs",
            sources.scene_project_splits_scene_asset,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout.rs",
            sources.scene_project_splits_split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/route.rs",
            sources.scene_project_splits_split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/source_inventory.rs",
            sources.scene_project_splits_split_layout_source_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/sources.rs",
            sources.scene_project_splits_split_layout_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/status_docs.rs",
            sources.scene_project_splits_split_layout_status_docs,
        ),
    ] {
        super::assert_runtime_15_test_file_budget(path, source);
    }
}
