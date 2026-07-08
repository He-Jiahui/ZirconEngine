use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_scene_project_split_layout(sources: &SplitLayoutSources) {
    assert_scene_project_parent_route(sources);
    assert_scene_project_support_children(sources);
    assert_scene_project_split_route(sources);
    assert_scene_project_split_budgets(sources);
}

fn assert_scene_project_parent_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "scene/project route",
        sources.parent,
        &[
            "#[path = \"scene_project_splits/dynamic_session_event.rs\"]",
            "#[path = \"scene_project_splits/project_io.rs\"]",
            "#[path = \"scene_project_splits/scene_asset.rs\"]",
            "#[path = \"scene_project_splits/split_layout.rs\"]",
        ],
    );

    for moved_anchor in [
        "let scene_mod = include_str!",
        "let project_io_root = include_str!",
        "let session_root = include_str!",
        "fn occurrence_count",
        "for root_anchor in [",
        "for doc_anchor in [",
    ] {
        assert!(
            !sources.parent.contains(moved_anchor),
            "scene_project_splits.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }
}

fn assert_scene_project_support_children(sources: &SplitLayoutSources) {
    assert_contains_all(
        "scene asset child",
        sources.scene_asset,
        &[
            "runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
            "SceneMobilityAsset",
            "SceneSpotLightAsset",
        ],
    );
    assert_contains_all(
        "project I/O child",
        sources.project_io,
        &[
            "runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
            "project_io/camera.rs",
            "project_io/transform.rs",
        ],
    );
    assert_contains_all(
        "dynamic session event child",
        sources.dynamic_session_event,
        &[
            "runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
            "dynamic_api/session/events.rs",
            "ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1",
        ],
    );
}

fn assert_scene_project_split_route(sources: &SplitLayoutSources) {
    assert_contains_all(
        "scene/project split-layout route",
        sources.split_layout,
        &[
            "#[path = \"split_layout/route.rs\"]",
            "#[path = \"split_layout/source_inventory.rs\"]",
            "#[path = \"split_layout/sources.rs\"]",
            "#[path = \"split_layout/status_docs.rs\"]",
            "runtime_15_runtime_07_scene_project_guard_child_owner_split",
            "runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_split",
            "run_scene_project_split_layout_checks();",
            "route::assert_scene_project_split_layout(&sources);",
            "source_inventory::assert_scene_project_source_inventory(&sources);",
            "status_docs::assert_scene_project_split_docs(&sources);",
        ],
    );

    for moved_anchor in [
        "let parent = include_str!(\"../scene_project_splits.rs\")",
        "let source_inventory = include_str!",
        "for moved_anchor in [",
        "for (path, source) in [",
        "for (label, source) in [",
    ] {
        assert!(
            !sources.split_layout.contains(moved_anchor),
            "scene_project_splits/split_layout.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "scene/project split-layout children",
        &format!(
            "{}\n{}\n{}\n{}",
            sources.split_layout_route,
            sources.split_layout_source_inventory,
            sources.split_layout_sources,
            sources.split_layout_status_docs
        ),
        &[
            "assert_scene_project_split_layout",
            "assert_scene_project_source_inventory",
            "pub(super) struct SplitLayoutSources",
            "assert_scene_project_split_docs",
            "Runtime 15 M3 Runtime 07 scene/project split-layout guard folder-backed split",
        ],
    );
}

fn assert_scene_project_split_budgets(sources: &SplitLayoutSources) {
    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits.rs",
            sources.parent,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/scene_asset.rs",
            sources.scene_asset,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/project_io.rs",
            sources.project_io,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs",
            sources.dynamic_session_event,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout.rs",
            sources.split_layout,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/route.rs",
            sources.split_layout_route,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/source_inventory.rs",
            sources.split_layout_source_inventory,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/sources.rs",
            sources.split_layout_sources,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/status_docs.rs",
            sources.split_layout_status_docs,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 260,
            "{path} should stay below the focused scene/project split guard budget; got {line_count} lines"
        );
    }
}
