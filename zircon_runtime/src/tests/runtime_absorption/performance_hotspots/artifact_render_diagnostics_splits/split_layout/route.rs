use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_artifact_render_diagnostics_split_layout(sources: &SplitLayoutSources) {
    assert_artifact_render_diagnostics_parent_route(sources);
    assert_artifact_render_diagnostics_support_children(sources);
    assert_artifact_render_diagnostics_split_route(sources);
    assert_artifact_render_diagnostics_split_budgets(sources);
}

fn assert_artifact_render_diagnostics_parent_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "artifact/render diagnostics route",
        sources.parent,
        &[
            "#[path = \"artifact_render_diagnostics_splits/artifact_cache_payload.rs\"]",
            "#[path = \"artifact_render_diagnostics_splits/render_product_diagnostics.rs\"]",
            "#[path = \"artifact_render_diagnostics_splits/split_layout.rs\"]",
        ],
    );

    for moved_anchor in [
        "let cache_root = include_str!",
        "let product_root =",
        "for root_anchor in [",
        "for root_dispatch_anchor in [",
        "for moved_owner_anchor in [",
        "for doc_anchor in [",
    ] {
        assert!(
            !sources.parent.contains(moved_anchor),
            "artifact_render_diagnostics_splits.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }
}

fn assert_artifact_render_diagnostics_support_children(sources: &SplitLayoutSources) {
    assert_contains_all(
        "artifact cache payload child",
        sources.artifact_cache_payload,
        &[
            "runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
            "cache_payload/json_value.rs",
            "cache_payload/mesh.rs",
            "cache_payload/toml_value.rs",
        ],
    );
    assert_contains_all(
        "render product diagnostics child",
        sources.render_product_diagnostics,
        &[
            "runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
            "render_stats_store/product/camera.rs",
            "render_stats_store/product/gpu_scene.rs",
            "render_stats_store/product/ui.rs",
        ],
    );
}

fn assert_artifact_render_diagnostics_split_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "artifact/render diagnostics split-layout route",
        sources.split_layout,
        &[
            "#[path = \"split_layout/route.rs\"]",
            "#[path = \"split_layout/source_inventory.rs\"]",
            "#[path = \"split_layout/sources.rs\"]",
            "#[path = \"split_layout/status_docs.rs\"]",
            "runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split",
            "runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_split",
            "run_artifact_render_diagnostics_split_layout_checks();",
            "route::assert_artifact_render_diagnostics_split_layout(&sources);",
            "source_inventory::assert_artifact_render_diagnostics_source_inventory(&sources);",
            "status_docs::assert_artifact_render_diagnostics_split_docs(&sources);",
        ],
    );

    for moved_anchor in [
        "let parent = include_str!(\"../artifact_render_diagnostics_splits.rs\")",
        "let source_inventory = include_str!",
        "for moved_anchor in [",
        "for (path, source) in [",
        "for (label, source) in [",
    ] {
        assert!(
            !sources.split_layout.contains(moved_anchor),
            "artifact_render_diagnostics_splits/split_layout.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "artifact/render diagnostics split-layout children",
        &format!(
            "{}\n{}\n{}\n{}",
            sources.split_layout_route,
            sources.split_layout_source_inventory,
            sources.split_layout_sources,
            sources.split_layout_status_docs
        ),
        &[
            "assert_artifact_render_diagnostics_split_layout",
            "assert_artifact_render_diagnostics_source_inventory",
            "pub(super) struct SplitLayoutSources",
            "assert_artifact_render_diagnostics_split_docs",
            "Runtime 15 M3 Runtime 07 artifact/render diagnostics split-layout guard folder-backed split",
        ],
    );
}

fn assert_artifact_render_diagnostics_split_budgets(sources: &SplitLayoutSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits.rs",
            sources.parent,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/artifact_cache_payload.rs",
            sources.artifact_cache_payload,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/render_product_diagnostics.rs",
            sources.render_product_diagnostics,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout.rs",
            sources.split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/route.rs",
            sources.split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/source_inventory.rs",
            sources.split_layout_source_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/sources.rs",
            sources.split_layout_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout/status_docs.rs",
            sources.split_layout_status_docs,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 260,
            "{path} should stay below the focused artifact/render diagnostics split guard budget; got {line_count} lines"
        );
    }
}
