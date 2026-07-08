use super::super::sources::OwnerBudgetSources;

pub(super) fn assert_artifact_render_diagnostics_budgets(sources: &OwnerBudgetSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits.rs",
            sources.artifact_render_diagnostics,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/artifact_cache_payload.rs",
            sources.artifact_render_diagnostics_artifact_cache_payload,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/render_product_diagnostics.rs",
            sources.artifact_render_diagnostics_render_product_diagnostics,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout.rs",
            sources.artifact_render_diagnostics_split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/route.rs",
            sources.artifact_render_diagnostics_split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/source_inventory.rs",
            sources.artifact_render_diagnostics_split_layout_source_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/sources.rs",
            sources.artifact_render_diagnostics_split_layout_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/status_docs.rs",
            sources.artifact_render_diagnostics_split_layout_status_docs,
        ),
    ] {
        super::assert_runtime_15_test_file_budget(path, source);
    }
}
