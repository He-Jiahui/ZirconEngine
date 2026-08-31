use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_artifact_render_diagnostics_source_inventory(sources: &SplitLayoutSources) {
    assert_contains_all(
        "performance hotpath source inventory",
        sources.source_inventory,
        &[
            "RUNTIME_07_TEST_FILES = (",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/artifact_cache_payload.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/render_product_diagnostics.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/route.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/source_inventory.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/sources.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/status_docs.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
        ],
    );
}
